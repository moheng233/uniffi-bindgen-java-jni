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
}

/// Data for a single JNI bridge function.
#[derive(Debug, Clone)]
struct BridgeFunction {
    name: String,
    jni_name: String,
    comment: String,
    has_rust_call_status: bool,
    arguments: Vec<BridgeArg>,
    return_type: Option<String>,
}

/// Template for jni_bridge.rs
#[derive(Template)]
#[template(escape = "none", path = "rust/jni_bridge.rs")]
struct JniBridgeTemplate {
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
) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    let glue_crate_name = "uniffi-jni-glue";
    let cargo_toml_path = out_dir.join("Cargo.toml");
    let src_dir = out_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Generate Cargo.toml
    let cargo_toml = cargo_toml::generate_cargo_toml(glue_crate_name, root);
    fs::write(&cargo_toml_path, cargo_toml)?;

    // Generate lib.rs
    let lib_rs = generate_lib_rs(root, crate_filter)?;
    fs::write(src_dir.join("lib.rs"), lib_rs)?;

    // Generate jni_bridge.rs
    let jni_bridge = generate_jni_bridge(root, crate_filter)?;
    fs::write(src_dir.join("jni_bridge.rs"), jni_bridge)?;

    // Generate jni_types.rs
    let jni_types_rs = jni_types::generate_jni_types();
    fs::write(src_dir.join("jni_types.rs"), jni_types_rs)?;

    // Generate jni_callback.rs
    let jni_callback = callback_gen::generate_jni_callback(root, crate_filter)?;
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
                let args: Vec<BridgeArg> = func
                    .arguments
                    .iter()
                    .map(|a| BridgeArg {
                        name: a.name.clone(),
                        rust_type: ffi_type_rust_name(&a.ty),
                    })
                    .collect();

                let return_type = func.return_type.as_ref().map(ffi_type_rust_name);

                functions.push(BridgeFunction {
                    name: func.name.clone(),
                    jni_name: func.jni_name.clone(),
                    comment: format!("JNI bridge for {}", func.name),
                    has_rust_call_status: func.has_rust_call_status_arg,
                    arguments: args,
                    return_type,
                });
            }
        }
    }

    let tmpl = JniBridgeTemplate { functions };
    Ok(tmpl.render()?)
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

