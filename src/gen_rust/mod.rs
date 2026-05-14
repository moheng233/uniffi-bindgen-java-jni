//! Rust JNI glue code generation module.
//!
//! This module generates the Rust JNI glue crate that bridges
//! between Java native methods and UniFFI FFI functions.

use anyhow::Result;
use askama::Template;
use camino::Utf8Path;
use fs_err as fs;

use crate::pipeline::nodes::*;

mod cargo_toml;
mod jni_types;
mod callback_gen;
mod jni_func;

/// Data for a single JNI bridge function argument.
#[derive(Debug, Clone)]
struct BridgeArg {
    name: String,
    /// The JNI type in the function signature (e.g., "jint", "jlong", "jobject")
    jni_type: String,
    /// The Rust FFI type that the actual FFI function expects (e.g., "u32", "u64", "RustBuffer")
    ffi_rust_type: String,
    /// Pre-computed expression to convert from JNI value to FFI value.
    /// For primitives: `arg_name as u32`
    /// For buffers: `jni_bytebuffer_to_rustbuffer(&mut env, arg_name)`
    /// For handles: `arg_name as u64`
    conv_expr: String,
}

/// Data for a single JNI bridge function.
#[derive(Debug, Clone)]
struct BridgeFunction {
    jni_name: String,
    ffi_name: String,
    has_rust_call_status: bool,
    args: Vec<BridgeArg>,
    /// Whether this function needs `&mut env` (for JNI buffer conversions).
    /// Controls whether `env` is declared as `mut` and used (vs `_env`).
    needs_env: bool,
    /// The JNI return type (e.g., "jint", "jlong", "jobject")
    return_jni_type: Option<String>,
    /// The Rust FFI return type (e.g., "u32", "u64", "RustBuffer")
    return_ffi_rust_type: Option<String>,
    /// Pre-computed expression to convert from FFI return value to JNI value.
    /// For primitives: `result as jint`
    /// For buffers: `rustbuffer_to_jni_bytebuffer(&mut env, result)`
    return_conv_expr: Option<String>,
    /// Whether the return conversion expression needs an `unsafe` block.
    /// True for buffer conversions (rustbuffer_to_jni_bytebuffer is unsafe fn).
    /// False for primitive casts.
    return_conv_unsafe: bool,
}

/// Template for jni_bridge.rs
#[derive(Template)]
#[template(escape = "none", path = "rust/jni_bridge.rs")]
struct JniBridgeTemplate<'a> {
    main_crate_name: &'a str,
    functions: Vec<BridgeFunction>,
}

/// Template for lib.rs
#[derive(Template)]
#[template(escape = "none", path = "rust/lib.rs")]
struct LibTemplate {
    has_callbacks: bool,
    #[allow(dead_code)]
    modules: Vec<LibModule>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LibModule {
    name: String,
    bridge_fns: Vec<String>,
}

/// Generate Rust JNI glue code from the Java IR root node.
pub fn generate_rust_glue(
    root: &Root,
    out_dir: &Utf8Path,
    crate_filter: Option<&str>,
    main_crate_path: Option<&Utf8Path>,
) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    // Use the cdylib_name from the first module as the glue crate name,
    // so that `cargo build` produces a DLL with the name that Java's
    // System.loadLibrary() expects.
    let glue_crate_name = root.modules.values()
        .next()
        .map(|m| m.cdylib_name.as_str())
        .unwrap_or("uniffi-jni-glue");
    let cargo_toml_path = out_dir.join("Cargo.toml");
    let src_dir = out_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Convert main_crate_path to a path relative to out_dir
    let main_crate_rel: Option<camino::Utf8PathBuf> = match main_crate_path {
        Some(p) => {
            let p_abs = canonicalize_stripped(p)?;
            let out_abs = canonicalize_stripped(out_dir)?;
            // Use forward slashes for TOML compatibility
            let rel = path_relative_to(&p_abs, &out_abs);
            Some(camino::Utf8PathBuf::from(rel.as_str().replace('\\', "/")))
        }
        None => None,
    };

    // Generate Cargo.toml (trim trailing whitespace to avoid TOML parse errors)
    let mut cargo_toml = cargo_toml::generate_cargo_toml(
        glue_crate_name,
        root,
        main_crate_rel.as_deref(),
    );
    // Normalize line endings and trim trailing whitespace
    cargo_toml = cargo_toml.trim_end().to_string() + "\n";
    fs::write(&cargo_toml_path, &cargo_toml)?;

    // Generate lib.rs
    let lib_rs = normalize_content(&generate_lib_rs(root, crate_filter)?);
    fs::write(src_dir.join("lib.rs"), lib_rs)?;

    // Generate jni_bridge.rs
    let jni_bridge = normalize_content(&generate_jni_bridge(root, crate_filter)?);
    fs::write(src_dir.join("jni_bridge.rs"), jni_bridge)?;

    // Generate jni_types.rs
    let jni_types_rs = normalize_content(&jni_types::generate_jni_types());
    fs::write(src_dir.join("jni_types.rs"), jni_types_rs)?;

    // Generate jni_callback.rs
    let jni_callback = normalize_content(&callback_gen::generate_jni_callback(root, crate_filter)?);
    fs::write(src_dir.join("jni_callback.rs"), jni_callback)?;

    println!("Generated Rust glue crate at: {}", out_dir);

    Ok(())
}

fn generate_lib_rs(root: &Root, crate_filter: Option<&str>) -> Result<String> {
    let mut modules = Vec::new();

    let has_callbacks = root.modules.values().any(|m| {
        if let Some(filter) = crate_filter
            && m.crate_name != filter {
                return false;
            }
        m.has_callback_interface
    });

    for module in root.modules.values() {
        if let Some(filter) = crate_filter
            && module.crate_name != filter {
                continue;
        }
        let bridge_fns: Vec<String> = module
            .ffi_definitions
            .iter()
            .filter_map(|ffi| match ffi {
                FfiDefinition::RustFunction(func) => Some(func.name.clone()),
                _ => None,
            })
            .collect();
        modules.push(LibModule {
            name: module.name.clone(),
            bridge_fns,
        });
    }

    let tmpl = LibTemplate {
        has_callbacks,
        modules,
    };
    Ok(tmpl.render()?)
}

fn generate_jni_bridge(root: &Root, crate_filter: Option<&str>) -> Result<String> {
    let mut functions = Vec::new();

    for module in root.modules.values() {
        if let Some(filter) = crate_filter
            && module.crate_name != filter {
                continue;
            }

        for ffi in &module.ffi_definitions {
            if let FfiDefinition::RustFunction(func) = ffi {
                // Skip future-related FFI functions if the module has no async functions
                if module.async_fn_count == 0
                    && (func.name.contains("rust_future_") || func.name.contains("rustfuture"))
                {
                    continue;
                }

                // Skip callback init functions (VTable types not yet implemented)
                if func.name.contains("_init_callback_vtable_") {
                    continue;
                }

                let args: Vec<BridgeArg> = func
                    .arguments
                    .iter()
                    .map(bridge_arg_for_ffi)
                    .collect();

                let (return_jni_type, return_ffi_rust_type, return_conv_expr) =
                    match &func.return_type {
                        Some(ty) => {
                            let jni = ffi_type_to_jni_name(ty);
                            let ffi = ffi_type_to_ffi_rust_name(ty);
                            let conv = return_conv_for_ffi(ty);
                            (Some(jni), Some(ffi), Some(conv))
                        }
                        None => (None, None, None),
                    };

                // Determine if this function needs &mut env
                let needs_env = args.iter().any(|a| a.conv_expr.contains("&mut env"))
                    || return_conv_expr.as_deref().is_some_and(|e| e.contains("&mut env"));

                // Determine if the return conversion needs an unsafe block
                let return_conv_unsafe = return_conv_expr
                    .as_deref()
                    .is_some_and(|e| e.contains("rustbuffer_to_jni_bytebuffer"));

                functions.push(BridgeFunction {
                    jni_name: func.jni_name.clone(),
                    ffi_name: func.name.clone(),
                    has_rust_call_status: func.has_rust_call_status_arg,
                    args,
                    needs_env,
                    return_jni_type,
                    return_ffi_rust_type,
                    return_conv_expr,
                    return_conv_unsafe,
                });
            }
        }
    }

    let tmpl = JniBridgeTemplate {
        main_crate_name: get_main_crate_name(root),
        functions,
    };
    Ok(tmpl.render()?)
}

/// Get the main crate name for Rust `use` import (underscore form).
fn get_main_crate_name(root: &Root) -> &str {
    root.modules
        .values()
        .next()
        .map(|m| m.crate_name.as_str())
        .unwrap_or("main_crate")
}

/// Map a Java IR FfiType to the JNI type used in the `extern "system" fn` signature.
fn ffi_type_to_jni_name(ty: &FfiType) -> String {
    match ty {
        FfiType::Int8 | FfiType::UInt8 => "jbyte".into(),
        FfiType::Int16 | FfiType::UInt16 => "jshort".into(),
        FfiType::Int32 | FfiType::UInt32 => "jint".into(),
        FfiType::Int64 | FfiType::UInt64 => "jlong".into(),
        FfiType::Float32 => "jfloat".into(),
        FfiType::Float64 => "jdouble".into(),
        FfiType::Boolean => "jboolean".into(),
        FfiType::String | FfiType::Bytes | FfiType::RustBuffer | FfiType::ForeignBytes
        | FfiType::RustArc | FfiType::VoidPointer => "jobject".into(),
        FfiType::Handle => "jlong".into(),
        FfiType::Function(_) | FfiType::Callback(_) | FfiType::Reference(_) => "jlong".into(),
        FfiType::Struct(_) => "jobject".into(),
    }
}

/// Map a Java IR FfiType to the actual Rust type used in the FFI function signature.
fn ffi_type_to_ffi_rust_name(ty: &FfiType) -> String {
    match ty {
        FfiType::Int8 => "i8".into(),
        FfiType::UInt8 => "u8".into(),
        FfiType::Int16 => "i16".into(),
        FfiType::UInt16 => "u16".into(),
        FfiType::Int32 => "i32".into(),
        FfiType::UInt32 => "u32".into(),
        FfiType::Int64 => "i64".into(),
        FfiType::UInt64 => "u64".into(),
        FfiType::Float32 => "f32".into(),
        FfiType::Float64 => "f64".into(),
        FfiType::Boolean => "u8".into(),
        FfiType::String | FfiType::Bytes | FfiType::RustBuffer => "RustBuffer".into(),
        FfiType::ForeignBytes => "ForeignBytes".into(),
        FfiType::Handle => "uniffi::Handle".into(),
        FfiType::RustArc | FfiType::VoidPointer => "*const std::ffi::c_void".into(),
        FfiType::Function(name) => name.clone(),
        FfiType::Struct(name) => name.clone(),
        FfiType::Callback(name) => name.clone(),
        FfiType::Reference(inner) => format!("*const {}", ffi_type_to_ffi_rust_name(inner)),
    }
}

/// Build a BridgeArg from an FfiArgument, computing the JNI type, FFI type, and conversion expression.
fn bridge_arg_for_ffi(arg: &FfiArgument) -> BridgeArg {
    let jni_type = ffi_type_to_jni_name(&arg.ty);
    let ffi_rust_type = ffi_type_to_ffi_rust_name(&arg.ty);
    let conv_expr = arg_conv_expr(&arg.name, &arg.ty);
    BridgeArg {
        name: arg.name.clone(),
        jni_type,
        ffi_rust_type,
        conv_expr,
    }
}

/// Compute the conversion expression from JNI parameter to FFI argument.
fn arg_conv_expr(arg_name: &str, ty: &FfiType) -> String {
    match ty {
        // Primitives: cast from JNI type to Rust FFI type
        FfiType::Int8 => format!("{arg_name} as i8"),
        FfiType::UInt8 => format!("{arg_name} as u8"),
        FfiType::Int16 => format!("{arg_name} as i16"),
        FfiType::UInt16 => format!("{arg_name} as u16"),
        FfiType::Int32 => format!("{arg_name} as i32"),
        FfiType::UInt32 => format!("{arg_name} as u32"),
        FfiType::Int64 => format!("{arg_name} as i64"),
        FfiType::UInt64 => format!("{arg_name} as u64"),
        FfiType::Float32 => format!("{arg_name} as f32"),
        FfiType::Float64 => format!("{arg_name} as f64"),
        FfiType::Boolean => format!("{arg_name} as u8"),
        // Buffers: convert via helper (jni functions are unsafe fn)
        FfiType::String | FfiType::Bytes | FfiType::RustBuffer => {
            format!("unsafe {{ jni_bytebuffer_to_rustbuffer(&mut env, {arg_name}) }}")
        }
        // ForeignBytes: read ByteBuffer data, create ForeignBytes (jni function is unsafe fn)
        FfiType::ForeignBytes => {
            format!("unsafe {{ jni_bytebuffer_to_foreignbytes(&mut env, {arg_name}) }}")
        }
        // Handle: from_raw is unsafe (constructs Handle from raw value)
        FfiType::Handle => format!("unsafe {{ uniffi::Handle::from_raw({arg_name} as u64).expect(\"invalid handle\") }}"),
        // Pointers: cast jlong to pointer
        FfiType::RustArc | FfiType::VoidPointer | FfiType::Reference(_) => {
            format!("{arg_name} as *const std::ffi::c_void")
        }
        // Function/Callback: cast jlong
        FfiType::Function(_) | FfiType::Callback(_) => {
            format!("{arg_name} as usize")
        }
        // Struct: pass through
        FfiType::Struct(_) => arg_name.to_string(),
    }
}

/// Compute the return conversion expression from FFI return value to JNI return value.
fn return_conv_for_ffi(ty: &FfiType) -> String {
    match ty {
        FfiType::Int8 | FfiType::UInt8 => "result as jbyte".into(),
        FfiType::Int16 | FfiType::UInt16 => "result as jshort".into(),
        FfiType::Int32 | FfiType::UInt32 => "result as jint".into(),
        FfiType::Int64 | FfiType::UInt64 => "result as jlong".into(),
        FfiType::Float32 => "result as jfloat".into(),
        FfiType::Float64 => "result as jdouble".into(),
        FfiType::Boolean => "result as jboolean".into(),
        FfiType::String | FfiType::Bytes | FfiType::RustBuffer => {
            "rustbuffer_to_jni_bytebuffer(&mut env, result)".into()
        }
        FfiType::ForeignBytes => "result".into(),
        FfiType::Handle => "result.as_raw() as jlong".into(),
        FfiType::RustArc | FfiType::VoidPointer | FfiType::Function(_)
        | FfiType::Struct(_) | FfiType::Callback(_) | FfiType::Reference(_) => {
            "result".into()
        }
    }
}

/// Canonicalize a path, stripping the Windows `\\?\` prefix.
fn canonicalize_stripped(path: &camino::Utf8Path) -> Result<camino::Utf8PathBuf> {
    let abs = path.canonicalize_utf8()?;
    // Strip \\?\ prefix on Windows
    if let Ok(rel) = abs.strip_prefix(r"\\?\") {
        Ok(rel.into())
    } else {
        Ok(abs)
    }
}

/// Compute a relative path from `base` to `target`.
fn path_relative_to(target: &camino::Utf8Path, base: &camino::Utf8Path) -> camino::Utf8PathBuf {
    let mut target_components = target.components().peekable();
    let mut base_components = base.components().peekable();

    // Skip common prefix
    while let (Some(tc), Some(bc)) = (target_components.peek(), base_components.peek()) {
        if tc == bc {
            target_components.next();
            base_components.next();
        } else {
            break;
        }
    }

    // Add .. for each remaining component in base
    let mut result = camino::Utf8PathBuf::new();
    for _ in base_components {
        result.push("..");
    }
    // Add remaining target components
    for c in target_components {
        result.push(c.as_str());
    }

    if result.as_str().is_empty() {
        result.push(".");
    }
    result
}

/// Normalize generated content: trim trailing whitespace and ensure a single trailing newline.
fn normalize_content(content: &str) -> String {
    let trimmed = content.trim_end();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}\n", trimmed)
    }
}

