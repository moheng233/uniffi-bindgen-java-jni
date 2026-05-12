//! Askama filter functions for Java JNI templates.
//!
//! These filters convert Rust pipeline types into Java source code snippets.

use askama::Result as AskamaResult;

use super::nodes::*;

/// Convert an FfiType to a Java JNI type string for native method declarations.
///
/// JNI mappings:
/// - jbyte (Int8), jshort (Int16), jint (Int32), jlong (Int64)
/// - jfloat (Float32), jdouble (Float64)
/// - jobject (String, ByteBuffer, etc.)
#[askama::filter_fn]
pub fn ffi_type_java(ty: &FfiType, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(match ty {
        FfiType::Int8 => "byte".to_string(),
        FfiType::UInt8 => "byte".to_string(),
        FfiType::Int16 => "short".to_string(),
        FfiType::UInt16 => "short".to_string(),
        FfiType::Int32 => "int".to_string(),
        FfiType::UInt32 => "int".to_string(),
        FfiType::Int64 => "long".to_string(),
        FfiType::UInt64 => "long".to_string(),
        FfiType::Float32 => "float".to_string(),
        FfiType::Float64 => "double".to_string(),
        FfiType::Handle => "long".to_string(),
        FfiType::RustBuffer => "java.nio.ByteBuffer".to_string(),
        FfiType::RustArc => "long".to_string(),
        FfiType::VoidPointer => "long".to_string(),
        FfiType::Function(_) => "long".to_string(),
        FfiType::Struct(_) => "long".to_string(),
        FfiType::Callback(_) => "long".to_string(),
        FfiType::Reference(_) => "long".to_string(),
        _ => "java.lang.Object".to_string(),
    })
}

/// Convert a TypeNode to its Java type name.
#[askama::filter_fn]
pub fn java_type(ty: &TypeNode, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(java_type_inner(ty))
}

fn java_type_inner(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 => "byte".to_string(),
        TypeNode::UInt8 => "short".to_string(),
        TypeNode::Int16 => "short".to_string(),
        TypeNode::UInt16 => "int".to_string(),
        TypeNode::Int32 => "int".to_string(),
        TypeNode::UInt32 => "long".to_string(),
        TypeNode::Int64 => "long".to_string(),
        TypeNode::UInt64 => "long".to_string(),
        TypeNode::Float32 => "float".to_string(),
        TypeNode::Float64 => "double".to_string(),
        TypeNode::Boolean => "boolean".to_string(),
        TypeNode::String => "java.lang.String".to_string(),
        TypeNode::Bytes => "byte[]".to_string(),
        TypeNode::Timestamp => "java.time.Instant".to_string(),
        TypeNode::Duration => "java.time.Duration".to_string(),
        TypeNode::Object { name, .. } => name.clone(),
        TypeNode::CallbackInterface { name, .. } => name.clone(),
        TypeNode::Record { name, .. } => name.clone(),
        TypeNode::Enum { name, .. } => name.clone(),
        TypeNode::Optional(inner) => format!("java.util.Optional<{}>", java_type_inner(inner)),
        TypeNode::Sequence(inner) => format!("java.util.List<{}>", java_type_inner(inner)),
        TypeNode::Map(inner) => format!("java.util.Map<java.lang.String, {}>", java_type_inner(inner)),
        TypeNode::Custom { name, .. } => name.clone(),
        TypeNode::External { name, .. } => name.clone(),
    }
}
