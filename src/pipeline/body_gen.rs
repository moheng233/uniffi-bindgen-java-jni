//! Body generation for Java methods.
//!
//! Generates the Java code for method bodies (lower args → call native → lift result).
//! This logic is complex and better handled in Rust than in Askama templates.
//!
//! TODO: Reduce pre-computed string dependency. Currently all Java code is
//! assembled as strings here and injected into templates via `{{ func.body }}`.
//! This makes the code style fragmented and hard to maintain. The ideal approach
//! is to let Askama templates handle type matching and code generation natively,
//! but this is limited by Askama's lack of support for tuple variants (e.g.,
//! `TypeNode::Optional(Box<TypeNode>)`) and nested match+filter patterns.
//! Future options: (1) upgrade Askama; (2) custom `#[derive(Template)]` render;
//! (3) switch to Tera/minijinja.

use super::nodes::*;

/// Generate the body of a top-level function (static method).
pub fn generate_function_body(
    func: &Function,
    class_name: &str,
) -> String {
    let mut lines = Vec::new();

    // Lower each argument
    for arg in &func.arguments {
        let code = lower_code_for_arg(arg);
        if !code.is_empty() {
            lines.push(code);
        }
    }

    // Build the native call
    let status_arg = if func.throws.is_some() { ", status" } else { "" };
    let args_str: Vec<String> = func.arguments.iter()
        .map(|a| a.native_expr.clone())
        .collect();
    let all_args = format!("{}{}", args_str.join(", "), status_arg);

    let ffi_name = func.ffi_func.name();

    // Return statement
    match &func.return_type {
        Some(ret) => {
            if is_buffer_type(ret) {
                let lift = lift_code_for_type(ret);
                lines.push(format!(
                    "ByteBuffer _resultBuf = {}.{}({});",
                    class_name, ffi_name, all_args
                ));
                if func.throws.is_some() {
                    lines.push("Helpers.ensureSuccess(status);".to_string());
                }
                lines.push(format!("return {};", lift));
            } else if is_handle_type(ret) {
                lines.push(format!(
                    "long _resultHandle = {}.{}({});",
                    class_name, ffi_name, all_args
                ));
                if func.throws.is_some() {
                    lines.push("Helpers.ensureSuccess(status);".to_string());
                }
                let ty_name = type_name(ret);
                lines.push(format!("return new {}(_resultHandle);", ty_name));
            } else {
                // Primitive or direct type
                let jty = java_type_str(ret);
                lines.push(format!(
                    "{} _result = {}.{}({});",
                    jty, class_name, ffi_name, all_args
                ));
                if func.throws.is_some() {
                    lines.push("Helpers.ensureSuccess(status);".to_string());
                }
                lines.push("return _result;".to_string());
            }
        }
        None => {
            lines.push(format!(
                "{}.{}({});",
                class_name, ffi_name, all_args
            ));
            if func.throws.is_some() {
                lines.push("Helpers.ensureSuccess(status);".to_string());
            }
        }
    }

    indent_lines(&lines, 8)
}

/// Generate the body of an Object constructor.
pub fn generate_constructor_body(
    ctor: &Constructor,
    class_name: &str,
    obj_java_name: &str,
) -> String {
    let mut lines = Vec::new();

    for arg in &ctor.arguments {
        let code = lower_code_for_arg(arg);
        if !code.is_empty() {
            lines.push(code);
        }
    }

    let status_arg = if ctor.throws.is_some() { ", status" } else { "" };
    let args_str: Vec<String> = ctor.arguments.iter()
        .map(|a| a.native_expr.clone())
        .collect();
    let all_args = format_args_with_status(&args_str, status_arg);

    let ffi_name = ctor.ffi_func.name();

    lines.push(format!(
        "long _handle = {}.{}({});",
        class_name, ffi_name, all_args
    ));
    if ctor.throws.is_some() {
        lines.push("Helpers.ensureSuccess(status);".to_string());
    }
    lines.push(format!("return new {}(_handle);", obj_java_name));

    indent_lines(&lines, 8)
}

/// Generate the body of an Object method.
pub fn generate_method_body(
    method: &Method,
    class_name: &str,
) -> String {
    let mut lines = Vec::new();

    for arg in &method.arguments {
        let code = lower_code_for_arg(arg);
        if !code.is_empty() {
            lines.push(code);
        }
    }

    let status_arg = if method.throws.is_some() { ", status" } else { "" };
    let mut args_str: Vec<String> = method.arguments.iter()
        .map(|a| a.native_expr.clone())
        .collect();
    // Prepend the handle as first arg
    args_str.insert(0, "this.handle".to_string());
    let all_args = format_args_with_status(&args_str, status_arg);

    let ffi_name = method.ffi_func.name();

    match &method.return_type {
        Some(ret) => {
            if is_buffer_type(ret) {
                let lift = lift_code_for_type(ret);
                lines.push(format!(
                    "ByteBuffer _resultBuf = {}.{}({});",
                    class_name, ffi_name, all_args
                ));
                if method.throws.is_some() {
                    lines.push("Helpers.ensureSuccess(status);".to_string());
                }
                lines.push(format!("return {};", lift));
            } else if is_handle_type(ret) {
                lines.push(format!(
                    "long _resultHandle = {}.{}({});",
                    class_name, ffi_name, all_args
                ));
                if method.throws.is_some() {
                    lines.push("Helpers.ensureSuccess(status);".to_string());
                }
                let ty_name = type_name(ret);
                lines.push(format!("return new {}(_resultHandle);", ty_name));
            } else {
                let jty = java_type_str(ret);
                lines.push(format!(
                    "{} _result = {}.{}({});",
                    jty, class_name, ffi_name, all_args
                ));
                if method.throws.is_some() {
                    lines.push("Helpers.ensureSuccess(status);".to_string());
                }
                lines.push("return _result;".to_string());
            }
        }
        None => {
            lines.push(format!(
                "{}.{}({});",
                class_name, ffi_name, all_args
            ));
            if method.throws.is_some() {
                lines.push("Helpers.ensureSuccess(status);".to_string());
            }
        }
    }

    indent_lines(&lines, 8)
}

// --- Helpers ---

/// Compute the lowering code for an argument.
/// Returns empty string for primitives (pass through).
pub fn lower_code_for_arg(arg: &Argument) -> String {
    match &arg.ty {
        TypeNode::Int8 | TypeNode::UInt8 | TypeNode::Int16 | TypeNode::UInt16
        | TypeNode::Int32 | TypeNode::UInt32 | TypeNode::Int64 | TypeNode::UInt64
        | TypeNode::Float32 | TypeNode::Float64 | TypeNode::Boolean => {
            String::new()
        }
        TypeNode::String => {
            format!("RustBuffer {}Buf = RustBuffer.allocFromString({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Bytes => {
            format!("RustBuffer {}Buf = RustBuffer.allocFromBytes({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Object { .. } => {
            format!("long {}Handle = {}.getHandle();",
                arg.java_name, arg.java_name)
        }
        TypeNode::Record { .. } | TypeNode::Enum { .. } => {
            format!("ByteBuffer {}Buf = {}.write();",
                arg.java_name, arg.java_name)
        }
        TypeNode::CallbackInterface { name, .. } => {
            format!("long {}Handle = FfiConverter{}.INSTANCE.lower({});",
                arg.java_name, name, arg.java_name)
        }
        TypeNode::Optional(_) => {
            format!("ByteBuffer {}Buf = FfiConverterOptional.write({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Sequence(_) => {
            format!("ByteBuffer {}Buf = FfiConverterSequence.write({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Map(_) => {
            format!("ByteBuffer {}Buf = FfiConverterMap.write({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Timestamp => {
            format!("ByteBuffer {}Buf = FfiConverterTimestamp.write({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Duration => {
            format!("ByteBuffer {}Buf = FfiConverterDuration.write({});",
                arg.java_name, arg.java_name)
        }
        TypeNode::Custom { name, .. } => {
            format!("ByteBuffer {}Buf = FfiConverter{}.write({});",
                arg.java_name, name, arg.java_name)
        }
        TypeNode::External { name, .. } => {
            format!("long {}Handle = FfiConverter{}.INSTANCE.lower({});",
                arg.java_name, name, arg.java_name)
        }
    }
}

/// Compute the native expression for an argument (the value to pass in the native call).
pub fn native_expr_for_arg(arg: &Argument) -> String {
    match &arg.ty {
        TypeNode::Int8 | TypeNode::Int16 | TypeNode::Int32 | TypeNode::Int64
        | TypeNode::UInt64 | TypeNode::Float32 | TypeNode::Float64 | TypeNode::Boolean => {
            arg.java_name.clone()
        }
        TypeNode::UInt8 => format!("(byte) {}", arg.java_name),
        TypeNode::UInt16 => format!("(short) {}", arg.java_name),
        TypeNode::UInt32 => format!("(int) {}", arg.java_name),
        TypeNode::Object { .. } | TypeNode::CallbackInterface { .. }
        | TypeNode::External { .. } => {
            format!("{}Handle", arg.java_name)
        }
        TypeNode::String | TypeNode::Bytes => {
            format!("{}Buf.asByteBuffer()", arg.java_name)
        }
        _ => {
            // String, Bytes, Record, Enum, Optional, Sequence, Map,
            // Timestamp, Duration, Custom — use the buffer variable
            format!("{}Buf", arg.java_name)
        }
    }
}

fn lift_code_for_type(ty: &TypeNode) -> String {
    match ty {
        TypeNode::String => {
            "RustBuffer.readStringFromByteBuffer(_resultBuf)".to_string()
        }
        TypeNode::Bytes => {
            "RustBuffer.readBytesFromByteBuffer(_resultBuf)".to_string()
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
        _ => "_result".to_string(),
    }
}

fn is_buffer_type(ty: &TypeNode) -> bool {
    matches!(ty,
        TypeNode::String | TypeNode::Bytes
        | TypeNode::Record { .. } | TypeNode::Enum { .. }
        | TypeNode::Optional(_) | TypeNode::Sequence(_) | TypeNode::Map(_)
        | TypeNode::Timestamp | TypeNode::Duration
        | TypeNode::Custom { .. }
    )
}

fn is_handle_type(ty: &TypeNode) -> bool {
    matches!(ty,
        TypeNode::Object { .. }
        | TypeNode::CallbackInterface { .. }
        | TypeNode::External { .. }
    )
}

fn type_name(ty: &TypeNode) -> String {
    match ty {
        TypeNode::Object { name, .. } => name.clone(),
        TypeNode::CallbackInterface { name, .. } => name.clone(),
        TypeNode::Record { name, .. } => name.clone(),
        TypeNode::Enum { name, .. } => name.clone(),
        TypeNode::Custom { name, .. } => name.clone(),
        TypeNode::External { name, .. } => name.clone(),
        _ => "Object".to_string(),
    }
}

fn java_type_str(ty: &TypeNode) -> String {
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
        TypeNode::Object { name, .. } => name.clone(),
        TypeNode::Record { name, .. } => name.clone(),
        TypeNode::Enum { name, .. } => name.clone(),
        TypeNode::CallbackInterface { name, .. } => name.clone(),
        TypeNode::Timestamp => "java.time.Instant".to_string(),
        TypeNode::Duration => "java.time.Duration".to_string(),
        TypeNode::Optional(inner) => format!("java.util.Optional<{}>", java_type_str(inner)),
        TypeNode::Sequence(inner) => format!("java.util.List<{}>", java_type_str(inner)),
        TypeNode::Map(inner) => format!("java.util.Map<String, {}>", java_type_str(inner)),
        TypeNode::Custom { name, .. } => name.clone(),
        TypeNode::External { name, .. } => name.clone(),
    }
}

fn format_args_with_status(args: &[String], status: &str) -> String {
    if args.is_empty() {
        status.trim_start_matches(", ").to_string()
    } else {
        format!("{}{}", args.join(", "), status)
    }
}

/// Generate write() body for a Record type.
pub fn generate_record_write_body(rec: &Record) -> String {
    let mut lines = Vec::new();
    
    // Estimate allocation size
    let mut alloc_parts = Vec::new();
    for field in &rec.fields {
        alloc_parts.push(allocation_size_expr(&field.ty, &field.java_name));
    }
    let alloc_str = alloc_parts.join(" + ");
    lines.push(format!("int _size = {};", alloc_str));
    lines.push("ByteBuffer _buf = ByteBuffer.allocateDirect(_size);".to_string());
    lines.push("_buf.order(java.nio.ByteOrder.BIG_ENDIAN);".to_string());
    lines.push("RustBufferStream _stream = new RustBufferStream(_buf);".to_string());
    
    for field in &rec.fields {
        lines.push(write_field_expr(&field.ty, &field.java_name));
    }
    
    lines.push("_buf.flip();".to_string());
    lines.push("return _buf;".to_string());
    
    indent_lines(&lines, 8)
}

/// Generate read() body for a Record type.
pub fn generate_record_read_body(rec: &Record) -> String {
    let mut lines = Vec::new();
    
    lines.push("RustBufferStream _stream = new RustBufferStream(buf);".to_string());
    
    let mut field_reads = Vec::new();
    for field in &rec.fields {
        let (ty_decl, read_expr) = read_field_expr(&field.ty);
        field_reads.push((field.java_name.clone(), ty_decl, read_expr));
    }
    
    for (name, ty_decl, read_expr) in &field_reads {
        lines.push(format!("{} {} = {};", ty_decl, name, read_expr));
    }
    
    let ctor_args: Vec<String> = field_reads.iter().map(|(n, _, _)| n.clone()).collect();
    lines.push(format!("return new {}({});", rec.java_name, ctor_args.join(", ")));
    
    indent_lines(&lines, 8)
}

fn allocation_size_expr(ty: &TypeNode, var_name: &str) -> String {
    match ty {
        TypeNode::Int8 | TypeNode::UInt8 | TypeNode::Boolean => "1".to_string(),
        TypeNode::Int16 | TypeNode::UInt16 => "2".to_string(),
        TypeNode::Int32 | TypeNode::UInt32 | TypeNode::Float32 => "4".to_string(),
        TypeNode::Int64 | TypeNode::UInt64 | TypeNode::Float64 => "8".to_string(),
        TypeNode::String => format!("4 + {0}.getBytes(java.nio.charset.StandardCharsets.UTF_8).length", var_name),
        TypeNode::Bytes => format!("4 + {}.length", var_name),
        TypeNode::Object { .. } | TypeNode::CallbackInterface { .. }
        | TypeNode::External { .. } => "8".to_string(),
        TypeNode::Record { .. } | TypeNode::Enum { .. } => {
            format!("{}.write().remaining()", var_name)
        }
        _ => "128".to_string(), // fallback for compounds
    }
}

fn write_field_expr(ty: &TypeNode, var_name: &str) -> String {
    match ty {
        TypeNode::Int8 => format!("_stream.writeInt8({});", var_name),
        TypeNode::UInt8 => format!("_stream.writeUInt8({});", var_name),
        TypeNode::Int16 => format!("_stream.writeInt16({});", var_name),
        TypeNode::UInt16 => format!("_stream.writeUInt16({});", var_name),
        TypeNode::Int32 => format!("_stream.writeInt32({});", var_name),
        TypeNode::UInt32 => format!("_stream.writeUInt32({});", var_name),
        TypeNode::Int64 => format!("_stream.writeInt64({});", var_name),
        TypeNode::UInt64 => format!("_stream.writeUInt64({});", var_name),
        TypeNode::Float32 => format!("_stream.writeFloat32({});", var_name),
        TypeNode::Float64 => format!("_stream.writeFloat64({});", var_name),
        TypeNode::Boolean => format!("_stream.writeBoolean({});", var_name),
        TypeNode::String => format!("_stream.writeString({});", var_name),
        TypeNode::Bytes => format!("_stream.writeBytes({});", var_name),
        TypeNode::Object { .. } | TypeNode::CallbackInterface { .. }
        | TypeNode::External { .. } => format!("_stream.writeHandle({}.getHandle());", var_name),
        TypeNode::Record { .. } | TypeNode::Enum { .. } => {
            format!("_stream.writeBytes({}.write().array());", var_name)
        }
        _ => format!("// TODO: write field {}", var_name),
    }
}

fn read_field_expr(ty: &TypeNode) -> (String, String) {
    // Returns (type_declaration, read_expression)
    match ty {
        TypeNode::Int8 => ("byte".to_string(), "_stream.readInt8()".to_string()),
        TypeNode::UInt8 => ("short".to_string(), "_stream.readUInt8()".to_string()),
        TypeNode::Int16 => ("short".to_string(), "_stream.readInt16()".to_string()),
        TypeNode::UInt16 => ("int".to_string(), "_stream.readUInt16()".to_string()),
        TypeNode::Int32 => ("int".to_string(), "_stream.readInt32()".to_string()),
        TypeNode::UInt32 => ("long".to_string(), "_stream.readUInt32()".to_string()),
        TypeNode::Int64 => ("long".to_string(), "_stream.readInt64()".to_string()),
        TypeNode::UInt64 => ("long".to_string(), "_stream.readUInt64()".to_string()),
        TypeNode::Float32 => ("float".to_string(), "_stream.readFloat32()".to_string()),
        TypeNode::Float64 => ("double".to_string(), "_stream.readFloat64()".to_string()),
        TypeNode::Boolean => ("boolean".to_string(), "_stream.readBoolean()".to_string()),
        TypeNode::String => ("String".to_string(), "_stream.readString()".to_string()),
        TypeNode::Bytes => ("byte[]".to_string(), "_stream.readBytes()".to_string()),
        TypeNode::Object { name, .. } => (name.clone(), format!("new {}(_stream.readHandle())", name)),
        TypeNode::Record { name, .. } => (name.clone(), format!("{}.read(_stream.buffer())", name)),
        TypeNode::Enum { name, .. } => (name.clone(), format!("{}.read(_stream.buffer())", name)),
        TypeNode::CallbackInterface { name, .. } => (name.clone(), format!("FfiConverter{}.INSTANCE.lift(_stream.readHandle())", name)),
        _ => ("Object".to_string(), "null /* TODO */".to_string()),
    }
}

/// Generate write() body for an Enum type.
pub fn generate_enum_write_body(e: &Enum) -> String {
    let mut lines = Vec::new();
    
    if e.is_flat {
        // Flat enum: just write the variant index
        lines.push("// Switch on the enum variant".to_string());
        lines.push("if (false) {} // placeholder for variant dispatch".to_string());
        lines.push("int _size = 4;".to_string());
        lines.push("ByteBuffer _buf = ByteBuffer.allocateDirect(_size);".to_string());
        lines.push("_buf.order(java.nio.ByteOrder.BIG_ENDIAN);".to_string());
        lines.push("RustBufferStream _stream = new RustBufferStream(_buf);".to_string());
        // Will be replaced with proper variant dispatch
        lines.push("_stream.writeInt32(0); // TODO: actual variant index".to_string());
        lines.push("_buf.flip();".to_string());
        lines.push("return _buf;".to_string());
    } else {
        // Enum with data variants
        let mut variant_cases = Vec::new();
        for (i, variant) in e.variants.iter().enumerate() {
            let mut case_lines = Vec::new();
            for field in &variant.fields {
                case_lines.push(format!("int _size_{} = {};", i, allocation_size_expr(&field.ty, &field.java_name)));
            }
            variant_cases.push((variant.java_name.clone(), i, case_lines));
        }
        
        lines.push("// Compute allocation size".to_string());
        lines.push("int _size = 4; // variant index".to_string());
        lines.push("// TODO: compute per-variant field sizes".to_string());
        lines.push("ByteBuffer _buf = ByteBuffer.allocateDirect(_size + 64);".to_string());
        lines.push("_buf.order(java.nio.ByteOrder.BIG_ENDIAN);".to_string());
        lines.push("RustBufferStream _stream = new RustBufferStream(_buf);".to_string());
        lines.push("// Write variant dispatch".to_string());
        lines.push("// TODO: switch on variant type and write index + fields".to_string());
        lines.push("_buf.flip();".to_string());
        lines.push("return _buf;".to_string());
    }
    
    indent_lines(&lines, 8)
}

/// Generate read() body for an Enum type.
pub fn generate_enum_read_body(e: &Enum) -> String {
    let mut lines = Vec::new();
    
    lines.push("RustBufferStream _stream = new RustBufferStream(buf);".to_string());
    lines.push("int _variantIdx = _stream.readInt32();".to_string());
    
    for (i, variant) in e.variants.iter().enumerate() {
        if i == 0 {
            lines.push(format!("if (_variantIdx == {}) {{", i));
        } else {
            lines.push(format!("}} else if (_variantIdx == {}) {{", i));
        }
        
        if variant.fields.is_empty() {
            lines.push(format!("    return new {}.{}();", e.java_name, variant.java_name));
        } else {
            let mut field_reads = Vec::new();
            for field in &variant.fields {
                let (ty_decl, read_expr) = read_field_expr(&field.ty);
                field_reads.push((field.java_name.clone(), ty_decl, read_expr));
            }
            for (name, ty_decl, read_expr) in &field_reads {
                lines.push(format!("    {} {} = {};", ty_decl, name, read_expr));
            }
            let ctor_args: Vec<String> = field_reads.iter().map(|(n, _, _)| n.clone()).collect();
            lines.push(format!("    return new {}.{}({});", e.java_name, variant.java_name, ctor_args.join(", ")));
        }
    }
    lines.push("} else {".to_string());
    lines.push("    throw new RuntimeException(\"Unknown variant index: \" + _variantIdx);".to_string());
    lines.push("}".to_string());
    
    indent_lines(&lines, 8)
}

fn indent_lines(lines: &[String], spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
            result.push_str(&indent);
        }
        result.push_str(line);
    }
    result
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

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

    // ========== lower_code_for_arg tests ==========

    #[test]
    fn lower_primitive_returns_empty() {
        for ty in &[
            TypeNode::Int8, TypeNode::UInt8, TypeNode::Int16, TypeNode::UInt16,
            TypeNode::Int32, TypeNode::UInt32, TypeNode::Int64, TypeNode::UInt64,
            TypeNode::Float32, TypeNode::Float64, TypeNode::Boolean,
        ] {
            let arg = make_arg("x", ty.clone());
            assert_eq!(lower_code_for_arg(&arg), "", "type {:?} should have empty lower", ty);
        }
    }

    #[test]
    fn lower_string_produces_buffer_alloc() {
        let arg = make_arg("name", TypeNode::String);
        assert_eq!(
            lower_code_for_arg(&arg),
            "RustBuffer nameBuf = RustBuffer.allocFromString(name);"
        );
    }

    #[test]
    fn lower_bytes_produces_buffer_alloc() {
        let arg = make_arg("data", TypeNode::Bytes);
        assert_eq!(
            lower_code_for_arg(&arg),
            "RustBuffer dataBuf = RustBuffer.allocFromBytes(data);"
        );
    }

    #[test]
    fn lower_object_produces_handle() {
        let arg = make_arg("obj", TypeNode::Object { namespace: "ns".into(), name: "Obj".into() });
        assert_eq!(
            lower_code_for_arg(&arg),
            "long objHandle = obj.getHandle();"
        );
    }

    #[test]
    fn lower_record_produces_write() {
        let arg = make_arg("rec", TypeNode::Record { namespace: "ns".into(), name: "Rec".into() });
        assert_eq!(
            lower_code_for_arg(&arg),
            "ByteBuffer recBuf = rec.write();"
        );
    }

    #[test]
    fn lower_enum_produces_write() {
        let arg = make_arg("e", TypeNode::Enum { namespace: "ns".into(), name: "E".into() });
        assert_eq!(
            lower_code_for_arg(&arg),
            "ByteBuffer eBuf = e.write();"
        );
    }

    #[test]
    fn lower_callback_interface_produces_ffi_converter() {
        let arg = make_arg("cb", TypeNode::CallbackInterface { namespace: "ns".into(), name: "MyCb".into() });
        assert_eq!(
            lower_code_for_arg(&arg),
            "long cbHandle = FfiConverterMyCb.INSTANCE.lower(cb);"
        );
    }

    #[test]
    fn lower_optional_produces_ffi_converter() {
        let arg = make_arg("opt", TypeNode::Optional(Box::new(TypeNode::Int32)));
        assert_eq!(
            lower_code_for_arg(&arg),
            "ByteBuffer optBuf = FfiConverterOptional.write(opt);"
        );
    }

    // ========== native_expr_for_arg tests ==========

    #[test]
    fn native_expr_primitive_is_unchanged() {
        let arg = make_arg("count", TypeNode::Int32);
        assert_eq!(native_expr_for_arg(&arg), "count");
    }

    #[test]
    fn native_expr_unsigned_types_are_narrowed() {
        assert_eq!(native_expr_for_arg(&make_arg("a", TypeNode::UInt8)), "(byte) a");
        assert_eq!(native_expr_for_arg(&make_arg("b", TypeNode::UInt16)), "(short) b");
        assert_eq!(native_expr_for_arg(&make_arg("c", TypeNode::UInt32)), "(int) c");
    }

    #[test]
    fn native_expr_object_uses_handle_suffix() {
        let arg = make_arg("obj", TypeNode::Object { namespace: "ns".into(), name: "O".into() });
        assert_eq!(native_expr_for_arg(&arg), "objHandle");
    }

    #[test]
    fn native_expr_buffer_type_uses_buf_suffix() {
        let arg = make_arg("s", TypeNode::String);
        assert_eq!(native_expr_for_arg(&arg), "sBuf.asByteBuffer()");

        let arg = make_arg("r", TypeNode::Record { namespace: "ns".into(), name: "R".into() });
        assert_eq!(native_expr_for_arg(&arg), "rBuf");
    }
}
