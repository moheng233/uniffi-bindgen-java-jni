// Type mapping utilities for Java JNI bindings.
//
// Maps UniFFI types to their Java equivalents.

use super::nodes::TypeNode;

/// Get the Java type name for a TypeNode.
pub fn java_type_name(ty: &TypeNode) -> String {
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
        TypeNode::String => "String".to_string(),
        TypeNode::Bytes => "byte[]".to_string(),
        TypeNode::Timestamp => "java.time.Instant".to_string(),
        TypeNode::Duration => "java.time.Duration".to_string(),
        TypeNode::Object { name, .. } => name.clone(),
        TypeNode::CallbackInterface { name, .. } => name.clone(),
        TypeNode::Record { name, .. } => name.clone(),
        TypeNode::Enum { name, .. } => name.clone(),
        TypeNode::Optional(inner) => format!("java.util.Optional<{}>", java_type_name(inner)),
        TypeNode::Sequence(inner) => format!("java.util.List<{}>", java_type_name(inner)),
        TypeNode::Map(inner) => format!("java.util.Map<String, {}>", java_type_name(inner)),
        TypeNode::Custom { name, .. } => name.clone(),
        TypeNode::External { name, .. } => name.clone(),
    }
}

/// Get the boxed Java type name (for generics).
pub fn java_boxed_type_name(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 => "Byte".to_string(),
        TypeNode::UInt8 => "Short".to_string(),
        TypeNode::Int16 => "Short".to_string(),
        TypeNode::UInt16 => "Integer".to_string(),
        TypeNode::Int32 => "Integer".to_string(),
        TypeNode::UInt32 => "Long".to_string(),
        TypeNode::Int64 => "Long".to_string(),
        TypeNode::UInt64 => "Long".to_string(),
        TypeNode::Float32 => "Float".to_string(),
        TypeNode::Float64 => "Double".to_string(),
        TypeNode::Boolean => "Boolean".to_string(),
        _ => java_type_name(ty),
    }
}
