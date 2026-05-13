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
pub fn ffi_type_java_str(ty: &FfiType) -> String {
    match ty {
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
    }
}

#[askama::filter_fn]
pub fn ffi_type_java(ty: &FfiType, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(ffi_type_java_str(ty))
}

/// Convert a TypeNode to its Java type name.
#[askama::filter_fn]
pub fn java_type(ty: &TypeNode, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(java_type_str(ty))
}

pub fn java_type_str(ty: &TypeNode) -> String {
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
        TypeNode::Optional(inner) => format!("java.util.Optional<{}>", java_type_str(inner)),
        TypeNode::Sequence(inner) => format!("java.util.List<{}>", java_type_str(inner)),
        TypeNode::Map(inner) => format!("java.util.Map<java.lang.String, {}>", java_type_str(inner)),
        TypeNode::Custom { name, .. } => name.clone(),
        TypeNode::External { name, .. } => name.clone(),
    }
}

/// Returns the lowering code for a function argument.
pub fn lower_code_str(arg: &Argument) -> String {
    match &arg.ty {
        TypeNode::Int8 | TypeNode::UInt8 | TypeNode::Int16 | TypeNode::UInt16
        | TypeNode::Int32 | TypeNode::UInt32 | TypeNode::Int64 | TypeNode::UInt64
        | TypeNode::Float32 | TypeNode::Float64 | TypeNode::Boolean => {
            String::new()
        }
        TypeNode::String => {
            format!("ByteBuffer {}Buf = RustBuffer.allocFromString({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Bytes => {
            format!("ByteBuffer {}Buf = RustBuffer.allocFromBytes({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Object { .. } => {
            format!("long {}Handle = {}.getHandle()",
                arg.java_name, arg.java_name)
        }
        TypeNode::Record { .. } => {
            format!("ByteBuffer {}Buf = {}.write()",
                arg.java_name, arg.java_name)
        }
        TypeNode::Enum { .. } => {
            format!("ByteBuffer {}Buf = {}.write()",
                arg.java_name, arg.java_name)
        }
        TypeNode::CallbackInterface { .. } => {
            let ty_name = java_type_str(&arg.ty);
            format!("long {}Handle = FfiConverter{}.INSTANCE.lower({})",
                arg.java_name, ty_name, arg.java_name)
        }
        TypeNode::Optional(_) => {
            format!("ByteBuffer {}Buf = FfiConverterOptional.write({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Sequence(_) => {
            format!("ByteBuffer {}Buf = FfiConverterSequence.write({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Map(_) => {
            format!("ByteBuffer {}Buf = FfiConverterMap.write({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Timestamp => {
            format!("ByteBuffer {}Buf = FfiConverterTimestamp.write({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Duration => {
            format!("ByteBuffer {}Buf = FfiConverterDuration.write({})",
                arg.java_name, arg.java_name)
        }
        TypeNode::Custom { .. } => {
            let ty_name = java_type_str(&arg.ty);
            format!("ByteBuffer {}Buf = FfiConverter{}.write({})",
                arg.java_name, ty_name, arg.java_name)
        }
        TypeNode::External { .. } => {
            format!("long {}Handle = FfiConverter{}.INSTANCE.lower({})",
                arg.java_name, java_type_str(&arg.ty), arg.java_name)
        }
    }
}

/// For primitives, returns empty string (pass through).
/// For String/Bytes/complex types, returns the full lowering statement.
#[askama::filter_fn]
pub fn lower_code(arg: &Argument, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(lower_code_str(arg))
}

/// Returns the expression used to pass this argument to the native method.
pub fn native_arg_str(arg: &Argument) -> String {
    match &arg.ty {
        TypeNode::Int8 | TypeNode::UInt8 | TypeNode::Int16 | TypeNode::UInt16
        | TypeNode::Int32 | TypeNode::UInt32 | TypeNode::Int64 | TypeNode::UInt64
        | TypeNode::Float32 | TypeNode::Float64 | TypeNode::Boolean => {
            arg.java_name.clone()
        }
        TypeNode::Object { .. } | TypeNode::CallbackInterface { .. }
        | TypeNode::External { .. } => {
            format!("{}Handle", arg.java_name)
        }
        _ => {
            // String, Bytes, Record, Enum, Optional, Sequence, Map,
            // Timestamp, Duration, Custom → use the buffer variable
            format!("{}Buf", arg.java_name)
        }
    }
}

#[askama::filter_fn]
pub fn native_arg(arg: &Argument, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(native_arg_str(arg))
}

/// Returns the return category: "buffer", "handle", or "direct".
/// Used to decide how to process the native method's return value.
pub fn return_category_str(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 | TypeNode::UInt8 | TypeNode::Int16 | TypeNode::UInt16
        | TypeNode::Int32 | TypeNode::UInt32 | TypeNode::Int64 | TypeNode::UInt64
        | TypeNode::Float32 | TypeNode::Float64 | TypeNode::Boolean => "direct".to_string(),
        TypeNode::Object { .. } => "handle".to_string(),
        TypeNode::CallbackInterface { .. } => "handle".to_string(),
        _ => "buffer".to_string(), // String, Bytes, Record, Enum, compounds, etc.
    }
}

#[askama::filter_fn]
pub fn return_category(ty: &TypeNode, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(return_category_str(ty))
}

/// Returns the expression to lift a native return value.
pub fn lift_code_str(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Int8 | TypeNode::UInt8 | TypeNode::Int16 | TypeNode::UInt16
        | TypeNode::Int32 | TypeNode::UInt32 | TypeNode::Int64 | TypeNode::UInt64
        | TypeNode::Float32 | TypeNode::Float64 | TypeNode::Boolean => {
            "_result".to_string()
        }
        TypeNode::String => {
            "RustBuffer.readStringFromByteBuffer(_resultBuf)".to_string()
        }
        TypeNode::Bytes => {
            "RustBuffer.readBytesFromByteBuffer(_resultBuf)".to_string()
        }
        TypeNode::Object { .. } => {
            format!("new {}(_resultHandle)", java_type_str(ty))
        }
        TypeNode::Record { name, .. } => {
            format!("{}.read(_resultBuf)", name)
        }
        TypeNode::Enum { name, .. } => {
            format!("{}.read(_resultBuf)", name)
        }
        TypeNode::CallbackInterface { name, .. } => {
            format!("FfiConverter{}.INSTANCE.lift(_resultHandle)", name)
        }
        TypeNode::Optional(inner) => {
            format!("FfiConverterOptional.read(_resultBuf, {})", java_type_str(inner))
        }
        TypeNode::Sequence(inner) => {
            format!("FfiConverterSequence.read(_resultBuf, {})", java_type_str(inner))
        }
        TypeNode::Map(inner) => {
            format!("FfiConverterMap.read(_resultBuf, {})", java_type_str(inner))
        }
        TypeNode::Timestamp => {
            "FfiConverterTimestamp.read(_resultBuf)".to_string()
        }
        TypeNode::Duration => {
            "FfiConverterDuration.read(_resultBuf)".to_string()
        }
        TypeNode::Custom { name, .. } => {
            format!("FfiConverter{}.read(_resultBuf)", name)
        }
        TypeNode::External { name, .. } => {
            format!("FfiConverter{}.INSTANCE.lift(_resultHandle)", name)
        }
    }
}

#[askama::filter_fn]
pub fn lift_code(ty: &TypeNode, _: &dyn askama::Values) -> AskamaResult<String> {
    Ok(lift_code_str(ty))
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    // ========== ffi_type_java tests ==========

    #[test]
    fn ffi_type_java_primitives() {
        assert_eq!(ffi_type(&FfiType::Int8), "byte");
        assert_eq!(ffi_type(&FfiType::UInt8), "byte");
        assert_eq!(ffi_type(&FfiType::Int16), "short");
        assert_eq!(ffi_type(&FfiType::UInt16), "short");
        assert_eq!(ffi_type(&FfiType::Int32), "int");
        assert_eq!(ffi_type(&FfiType::UInt32), "int");
        assert_eq!(ffi_type(&FfiType::Int64), "long");
        assert_eq!(ffi_type(&FfiType::UInt64), "long");
        assert_eq!(ffi_type(&FfiType::Float32), "float");
        assert_eq!(ffi_type(&FfiType::Float64), "double");
    }

    #[test]
    fn ffi_type_java_special() {
        assert_eq!(ffi_type(&FfiType::Handle), "long");
        assert_eq!(ffi_type(&FfiType::RustBuffer), "java.nio.ByteBuffer");
        assert_eq!(ffi_type(&FfiType::RustArc), "long");
        assert_eq!(ffi_type(&FfiType::String), "java.lang.Object");
        assert_eq!(ffi_type(&FfiType::Boolean), "java.lang.Object");
    }

    fn ffi_type(ty: &FfiType) -> String {
        ffi_type_java_str(ty)
    }

    // ========== java_type tests ==========

    #[test]
    fn java_type_primitives() {
        assert_eq!(jtype(&TypeNode::Int8), "byte");
        assert_eq!(jtype(&TypeNode::UInt8), "short");
        assert_eq!(jtype(&TypeNode::Int16), "short");
        assert_eq!(jtype(&TypeNode::UInt16), "int");
        assert_eq!(jtype(&TypeNode::Int32), "int");
        assert_eq!(jtype(&TypeNode::UInt32), "long");
        assert_eq!(jtype(&TypeNode::Int64), "long");
        assert_eq!(jtype(&TypeNode::UInt64), "long");
        assert_eq!(jtype(&TypeNode::Float32), "float");
        assert_eq!(jtype(&TypeNode::Float64), "double");
        assert_eq!(jtype(&TypeNode::Boolean), "boolean");
    }

    #[test]
    fn java_type_string_and_bytes() {
        assert_eq!(jtype(&TypeNode::String), "java.lang.String");
        assert_eq!(jtype(&TypeNode::Bytes), "byte[]");
    }

    #[test]
    fn java_type_named_types() {
        assert_eq!(
            jtype(&TypeNode::Object { namespace: "test".into(), name: "MyObj".into() }),
            "MyObj"
        );
        assert_eq!(
            jtype(&TypeNode::Record { namespace: "test".into(), name: "MyRec".into() }),
            "MyRec"
        );
        assert_eq!(
            jtype(&TypeNode::Enum { namespace: "test".into(), name: "MyEnum".into() }),
            "MyEnum"
        );
    }

    #[test]
    fn java_type_compound() {
        assert_eq!(
            jtype(&TypeNode::Optional(Box::new(TypeNode::String))),
            "java.util.Optional<java.lang.String>"
        );
        assert_eq!(
            jtype(&TypeNode::Sequence(Box::new(TypeNode::Int32))),
            "java.util.List<int>"
        );
        assert_eq!(
            jtype(&TypeNode::Map(Box::new(TypeNode::Float64))),
            "java.util.Map<java.lang.String, double>"
        );
    }

    fn jtype(ty: &TypeNode) -> String {
        java_type_str(ty)
    }

    // ========== lower_code tests ==========

    fn make_arg(name: &str, ty: TypeNode) -> Argument {
        Argument {
            name: name.into(),
            java_name: name.into(),
            ty,
            optional: false,
            default: None,
            lower_code: String::new(),
            native_expr: String::new(),
        }
    }

    #[test]
    fn lower_code_primitive_is_empty() {
        let arg = make_arg("x", TypeNode::Int32);
        assert_eq!(lower(&arg), "");
    }

    #[test]
    fn lower_code_string() {
        let arg = make_arg("name", TypeNode::String);
        assert_eq!(lower(&arg), "ByteBuffer nameBuf = RustBuffer.allocFromString(name)");
    }

    #[test]
    fn lower_code_object() {
        let arg = make_arg("obj", TypeNode::Object { namespace: "ns".into(), name: "Obj".into() });
        assert_eq!(lower(&arg), "long objHandle = obj.getHandle()");
    }

    #[test]
    fn lower_code_record() {
        let arg = make_arg("rec", TypeNode::Record { namespace: "ns".into(), name: "Rec".into() });
        assert_eq!(lower(&arg), "ByteBuffer recBuf = rec.write()");
    }

    fn lower(arg: &Argument) -> String {
        lower_code_str(arg)
    }

    // ========== return_category tests ==========

    #[test]
    fn return_category_direct() {
        assert_eq!(cat(&TypeNode::Int32), "direct");
        assert_eq!(cat(&TypeNode::Float64), "direct");
        assert_eq!(cat(&TypeNode::Boolean), "direct");
    }

    #[test]
    fn return_category_handle() {
        assert_eq!(cat(&TypeNode::Object { namespace: "ns".into(), name: "Obj".into() }), "handle");
    }

    #[test]
    fn return_category_buffer() {
        assert_eq!(cat(&TypeNode::String), "buffer");
        assert_eq!(cat(&TypeNode::Record { namespace: "ns".into(), name: "Rec".into() }), "buffer");
    }

    fn cat(ty: &TypeNode) -> String {
        return_category_str(ty)
    }

    // ========== lift_code tests ==========

    #[test]
    fn lift_code_direct() {
        assert_eq!(lift(&TypeNode::Int32), "_result");
        assert_eq!(lift(&TypeNode::Boolean), "_result");
    }

    #[test]
    fn lift_code_string() {
        assert_eq!(lift(&TypeNode::String), "RustBuffer.readStringFromByteBuffer(_resultBuf)");
    }

    #[test]
    fn lift_code_object() {
        assert_eq!(
            lift(&TypeNode::Object { namespace: "ns".into(), name: "Obj".into() }),
            "new Obj(_resultHandle)"
        );
    }

    fn lift(ty: &TypeNode) -> String {
        lift_code_str(ty)
    }

    // ========== native_arg tests ==========

    #[test]
    fn native_arg_primitive_passthrough() {
        let arg = make_arg("val", TypeNode::Int64);
        assert_eq!(native(&arg), "val");
    }

    #[test]
    fn native_arg_object_handle() {
        let arg = make_arg("obj", TypeNode::Object { namespace: "ns".into(), name: "O".into() });
        assert_eq!(native(&arg), "objHandle");
    }

    #[test]
    fn native_arg_buffer_type() {
        let arg = make_arg("s", TypeNode::String);
        assert_eq!(native(&arg), "sBuf");
    }

    fn native(arg: &Argument) -> String {
        native_arg_str(arg)
    }
}
