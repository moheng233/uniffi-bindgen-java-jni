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
    rust_type: String,
    /// Whether this argument is a RustBuffer (needs conversion from ByteBuffer).
    is_buffer: bool,
}

/// Data for a single JNI bridge function.
#[derive(Debug, Clone)]
struct BridgeFunction {
    jni_name: String,
    ffi_name: String,
    has_rust_call_status: bool,
    arguments: Vec<BridgeArg>,
    return_type: Option<String>,
    /// Whether the return type is a RustBuffer (needs conversion to ByteBuffer).
    return_is_buffer: bool,
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

    let glue_crate_name = "uniffi-jni-glue";
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

                let args: Vec<BridgeArg> = func
                    .arguments
                    .iter()
                    .map(|a| {
                        let is_buf = matches!(a.ty, FfiType::RustBuffer | FfiType::String | FfiType::Bytes);
                        BridgeArg {
                            name: a.name.clone(),
                            rust_type: ffi_type_rust_name(&a.ty),
                            is_buffer: is_buf,
                        }
                    })
                    .collect();

                let return_type = func.return_type.as_ref().map(ffi_type_rust_name);
                let return_is_buffer = func.return_type.as_ref()
                    .map(|ty| matches!(ty, FfiType::RustBuffer | FfiType::String | FfiType::Bytes))
                    .unwrap_or(false);

                functions.push(BridgeFunction {
                    jni_name: func.jni_name.clone(),
                    ffi_name: func.name.clone(), // FFI function name from conversion
                    has_rust_call_status: func.has_rust_call_status_arg,
                    arguments: args,
                    return_type,
                    return_is_buffer,
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

/// Get the main crate name (hyphens replaced with underscores for Rust identifiers).
fn get_main_crate_name(root: &Root) -> &str {
    root.modules
        .values()
        .next()
        .map(|m| m.crate_name.as_str())
        .unwrap_or("main_crate")
}

/// Map an FFI type to its Rust type name for use in JNI bridge signatures.
fn ffi_type_rust_name(ty: &FfiType) -> String {
    match ty {
        FfiType::Int8 => "jbyte".into(),
        FfiType::UInt8 => "jbyte".into(),
        FfiType::Int16 => "jshort".into(),
        FfiType::UInt16 => "jshort".into(),
        FfiType::Int32 => "jint".into(),
        FfiType::UInt32 => "jint".into(),
        FfiType::Int64 => "jlong".into(),
        FfiType::UInt64 => "jlong".into(),
        FfiType::Float32 => "jfloat".into(),
        FfiType::Float64 => "jdouble".into(),
        FfiType::Boolean => "jboolean".into(),
        FfiType::String => "jstring".into(),
        FfiType::Bytes => "jobject".into(),
        FfiType::Handle => "jlong".into(),
        FfiType::RustBuffer => "jobject".into(),
        FfiType::RustArc => "jobject".into(),
        FfiType::VoidPointer => "jobject".into(),
        FfiType::Function(_) => "jlong".into(),
        FfiType::Struct(name) => name.clone(),
        FfiType::Callback(_) => "jlong".into(),
        FfiType::Reference(_) => "jlong".into(),
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

