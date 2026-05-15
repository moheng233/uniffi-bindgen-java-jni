//! JNI callback generation for the Rust glue code.

use anyhow::Result;
use askama::Template;
use crate::pipeline::nodes::*;

/// Data for a single callback interface method in the template.
#[derive(Debug, Clone)]
struct CbiMethodData {
    name: String,
    vtable_field_name: String,
    java_callback_name: String,
    has_return: bool,
    return_ffi_type: String,
    ffi_args: Vec<CbiFfiArg>,
    jni_arg_values: Vec<String>,
    /// Object-typed arg declarations to emit before the JValue array
    jni_object_decls: Vec<String>,
    jni_signature: String,
    jni_extract_ret: String,
}

#[derive(Debug, Clone)]
struct CbiFfiArg {
    name: String,
    ffi_type: String,
}

/// Data for a single callback interface in the template.
#[derive(Debug, Clone)]
struct CbiData {
    name: String,
    java_name: String,
    init_fn: String,
    #[allow(dead_code)]
    vtable_type_name: String,
    methods: Vec<CbiMethodData>,
}

/// Template for rendering jni_callback module.
#[derive(Template)]
#[template(escape = "none", path = "rust/jni_callback.rs")]
struct JniCallbackTemplate {
    #[allow(dead_code)]
    main_crate_name: String,
    has_callbacks: bool,
    callback_interfaces: Vec<CbiData>,
}

/// Generate the jni_callback module source.
pub fn generate_jni_callback(root: &Root, crate_filter: Option<&str>) -> Result<String> {
    let main_crate_name = root.modules.values().next()
        .map(|m| m.crate_name.clone())
        .unwrap_or_else(|| "main_crate".to_string());

    let mut callback_interfaces = Vec::new();

    for module in root.modules.values() {
        if let Some(filter) = crate_filter
            && module.crate_name != filter {
                continue;
        }

        for td in &module.type_definitions {
            if let TypeDefinition::CallbackInterface(cbi) = td {
                let methods: Vec<CbiMethodData> = cbi.methods.iter().map(|method| {
                    build_cbi_method_data(method, &cbi.java_name)
                }).collect();

                // Build VTable type name: "VTableCallbackInterface" + ModuleName + InterfaceName
                // The module name uses the name field (e.g. "simple") upper-camel-cased
                let module_name = to_upper_camel(&module.name);
                let interface_name = &cbi.java_name;
                let vtable_type_name = format!("VTableCallbackInterface{}{}", module_name, interface_name);

                callback_interfaces.push(CbiData {
                    name: cbi.name.clone(),
                    java_name: cbi.java_name.clone(),
                    init_fn: cbi.ffi_init_callback.name().to_string(),
                    vtable_type_name,
                    methods,
                });
            }
        }
    }

    let has_callbacks = !callback_interfaces.is_empty();

    let tmpl = JniCallbackTemplate {
        main_crate_name,
        has_callbacks,
        callback_interfaces,
    };
    Ok(tmpl.render()?)
}

fn to_upper_camel(s: &str) -> String {
    heck::ToUpperCamelCase::to_upper_camel_case(s)
}

/// Build template data for a callback interface method.
fn build_cbi_method_data(method: &Method, cbi_java_name: &str) -> CbiMethodData {
    let has_return = method.return_type.is_some();

    // Build FFI args from the method's arguments (converting Java types to FFI types)
    let ffi_args: Vec<CbiFfiArg> = method.arguments.iter().map(|arg| {
        let ffi_type = java_type_to_ffi_rust(&arg.ty);
        CbiFfiArg {
            name: arg.name.clone(),
            ffi_type,
        }
    }).collect();

    // Build JNI arg value expressions and object declarations
    let mut jni_arg_values = Vec::new();
    let mut jni_object_decls = Vec::new();
    for (i, arg) in method.arguments.iter().enumerate() {
        let (decl_opt, value_expr) = jni_call_arg_expr(i, &arg.name, &arg.ty);
        if let Some(decl) = decl_opt {
            jni_object_decls.push(decl);
        }
        jni_arg_values.push(value_expr);
    }

    // Build JNI method signature
    let jni_signature = build_jni_signature(method);

    // Build JNI return value extraction
    let jni_extract_ret = if let Some(ref ret_ty) = method.return_type {
        jni_extract_return_expr(ret_ty)
    } else {
        String::new()
    };

    let return_ffi_type = method.return_type.as_ref()
        .map(java_type_to_ffi_rust)
        .unwrap_or_else(|| "()".to_string());

    let vtable_field_name = method.name.clone();

    CbiMethodData {
        name: vtable_field_name.clone(),
        vtable_field_name,
        java_callback_name: format!("callback{}_{}", cbi_java_name, method.java_name),
        has_return,
        return_ffi_type,
        ffi_args,
        jni_arg_values,
        jni_object_decls,
        jni_signature,
        jni_extract_ret,
    }
}

/// Map a Java TypeNode to the Rust FFI type used in callback signatures.
fn java_type_to_ffi_rust(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 => "i8".into(),
        TypeNode::UInt8 => "u8".into(),
        TypeNode::Int16 => "i16".into(),
        TypeNode::UInt16 => "u16".into(),
        TypeNode::Int32 => "i32".into(),
        TypeNode::UInt32 => "u32".into(),
        TypeNode::Int64 => "i64".into(),
        TypeNode::UInt64 => "u64".into(),
        TypeNode::Float32 => "f32".into(),
        TypeNode::Float64 => "f64".into(),
        TypeNode::Boolean => "u8".into(),
        TypeNode::String | TypeNode::Bytes => "RustBuffer".into(),
        TypeNode::Object { .. } => "u64".into(),
        TypeNode::Record { .. } | TypeNode::Enum { .. } => "RustBuffer".into(),
        TypeNode::CallbackInterface { .. } => "u64".into(),
        TypeNode::Timestamp | TypeNode::Duration => "u64".into(),
        TypeNode::Optional(_) | TypeNode::Sequence(_) | TypeNode::Map(_)
        | TypeNode::Custom { .. } | TypeNode::External { .. } => "RustBuffer".into(),
    }
}

/// Generate a JNI call argument expression. Returns (object_declaration, jvalue_expression).
/// Object declarations are let bindings for JNI objects that need to outlive the JValue array.
fn jni_call_arg_expr(idx: usize, name: &str, ty: &TypeNode) -> (Option<String>, String) {
    match ty {
        TypeNode::Int8 | TypeNode::UInt8 => (None, format!("JValue::Byte({name} as jbyte)")),
        TypeNode::Int16 | TypeNode::UInt16 => (None, format!("JValue::Short({name} as jshort)")),
        TypeNode::Int32 | TypeNode::UInt32 => (None, format!("JValue::Int({name} as jint)")),
        TypeNode::Int64 | TypeNode::UInt64 => (None, format!("JValue::Long({name} as jlong)")),
        TypeNode::Float32 => (None, format!("JValue::Float({name} as jfloat)")),
        TypeNode::Float64 => (None, format!("JValue::Double({name} as jdouble)")),
        TypeNode::Boolean => (None, format!("JValue::Bool({name} != 0)")),
        TypeNode::String => {
            let var = format!("_jni_arg_{idx}");
            let decl = format!("let {var} = unsafe {{ jni::objects::JObject::from_raw(rustbuffer_to_jni_string(&mut *env, {name})) }};");
            (Some(decl), format!("JValue::Object(&{var})"))
        }
        TypeNode::Bytes | TypeNode::Record { .. } | TypeNode::Enum { .. } => {
            let var = format!("_jni_arg_{idx}");
            let decl = format!("let {var} = unsafe {{ jni::objects::JObject::from_raw(rustbuffer_to_jni_bytebuffer(&mut *env, {name})) }};");
            (Some(decl), format!("JValue::Object(&{var})"))
        }
        TypeNode::Object { .. } | TypeNode::CallbackInterface { .. } => {
            (None, format!("JValue::Long({name} as jlong)"))
        }
        _ => (None, format!("JValue::Long({name} as jlong)")),
    }
}

/// Build JNI method signature for the static callback dispatch method.
fn build_jni_signature(method: &Method) -> String {
    let args_sig: String = method.arguments.iter().map(|arg| {
        java_type_to_jni_sig(&arg.ty)
    }).collect();

    let return_sig = match &method.return_type {
        Some(ty) => java_type_to_jni_sig(ty),
        None => "V".to_string(),
    };

    // Static method: (J + args) -> return
    // First arg is always long (handle)
    format!("(J{}){}", args_sig, return_sig)
}

/// Map Java type to JNI signature character.
fn java_type_to_jni_sig(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 | TypeNode::UInt8 => "B".into(),
        TypeNode::Int16 | TypeNode::UInt16 => "S".into(),
        TypeNode::Int32 | TypeNode::UInt32 => "I".into(),
        TypeNode::Int64 | TypeNode::UInt64 => "J".into(),
        TypeNode::Float32 => "F".into(),
        TypeNode::Float64 => "D".into(),
        TypeNode::Boolean => "Z".into(),
        TypeNode::String => "Ljava/lang/String;".into(),
        TypeNode::Bytes | TypeNode::Record { .. } | TypeNode::Enum { .. }
        | TypeNode::Optional(_) | TypeNode::Sequence(_) | TypeNode::Map(_)
        | TypeNode::Custom { .. } | TypeNode::External { .. } => {
            "Ljava/nio/ByteBuffer;".into()
        }
        TypeNode::Object { name, .. } => format!("L{name};"),
        _ => "J".into(),
    }
}

/// Generate JNI return value extraction from JValue.
fn jni_extract_return_expr(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 | TypeNode::UInt8 => "result.byte().unwrap() as u8".into(),
        TypeNode::Int16 | TypeNode::UInt16 => "result.short().unwrap() as u16".into(),
        TypeNode::Int32 | TypeNode::UInt32 => "result.int().unwrap() as u32".into(),
        TypeNode::Int64 | TypeNode::UInt64 => "result.long().unwrap() as u64".into(),
        TypeNode::Float32 => "result.float().unwrap()".into(),
        TypeNode::Float64 => "result.double().unwrap()".into(),
        TypeNode::Boolean => "result.bool().unwrap() as u8".into(),
        TypeNode::String | TypeNode::Bytes | TypeNode::Record { .. }
        | TypeNode::Enum { .. } => "jni_bytebuffer_extract_rustbuffer(&mut env, result.l().unwrap())".into(),
        TypeNode::Object { .. } | TypeNode::CallbackInterface { .. } => {
            "result.long().unwrap() as u64".into()
        }
        _ => "result.long().unwrap() as u64".into(),
    }
}


