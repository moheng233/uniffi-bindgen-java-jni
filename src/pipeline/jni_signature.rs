// JNI method signature generation.

/// Generate a JNI method signature string.
/// Example: "(Ljava/lang/String;I)J" for (String, int) -> long
pub fn jni_signature(args: &[super::nodes::FfiType], ret: Option<&super::nodes::FfiType>) -> String {
    let mut sig = String::from("(");
    for arg in args {
        sig.push_str(&ffi_type_to_jni(arg));
    }
    sig.push(')');
    match ret {
        Some(ty) => sig.push_str(&ffi_type_to_jni(ty)),
        None => sig.push('V'),
    }
    sig
}

fn ffi_type_to_jni(ty: &super::nodes::FfiType) -> String {
    match ty {
        super::nodes::FfiType::Int8 => "B".to_string(),
        super::nodes::FfiType::UInt8 => "B".to_string(),
        super::nodes::FfiType::Int16 => "S".to_string(),
        super::nodes::FfiType::UInt16 => "S".to_string(),
        super::nodes::FfiType::Int32 => "I".to_string(),
        super::nodes::FfiType::UInt32 => "I".to_string(),
        super::nodes::FfiType::Int64 => "J".to_string(),
        super::nodes::FfiType::UInt64 => "J".to_string(),
        super::nodes::FfiType::Float32 => "F".to_string(),
        super::nodes::FfiType::Float64 => "D".to_string(),
        super::nodes::FfiType::Boolean => "Z".to_string(),
        super::nodes::FfiType::String => "Ljava/lang/String;".to_string(),
        super::nodes::FfiType::Bytes => "[B".to_string(),
        super::nodes::FfiType::Handle => "J".to_string(),
        super::nodes::FfiType::RustBuffer => "Ljava/nio/ByteBuffer;".to_string(),
        super::nodes::FfiType::ForeignBytes => "Ljava/nio/ByteBuffer;".to_string(),
        super::nodes::FfiType::RustArc => "J".to_string(),
        super::nodes::FfiType::VoidPointer => "J".to_string(),
        super::nodes::FfiType::Function(_) => "J".to_string(),
        super::nodes::FfiType::Struct(_) => "Ljava/nio/ByteBuffer;".to_string(),
        super::nodes::FfiType::Callback(_) => "J".to_string(),
        super::nodes::FfiType::Reference(inner) => ffi_type_to_jni(inner),
    }
}
