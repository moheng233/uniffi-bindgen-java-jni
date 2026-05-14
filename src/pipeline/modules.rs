use anyhow::Result;
use uniffi_bindgen::pipeline::general;

use super::body_gen;
use super::config::JavaConfig;
use super::nodes::*;

/// Convert a general::Namespace to a Java Module.
pub fn convert_namespace(
    namespace: &general::Namespace,
    config: &JavaConfig,
    _cdylib: Option<&str>,
) -> Result<Module> {
    let package_name = config.package_name(&namespace.name);
    let cdylib_name = config.cdylib_name(&namespace.name);

    let imports: Vec<String> = vec![
        "java.nio.ByteBuffer".to_string(),
        "java.nio.ByteOrder".to_string(),
        "java.nio.charset.StandardCharsets".to_string(),
        "java.time.Duration".to_string(),
        "java.time.Instant".to_string(),
        "java.util.ArrayList".to_string(),
        "java.util.HashMap".to_string(),
        "java.util.List".to_string(),
        "java.util.Map".to_string(),
        "java.util.Optional".to_string(),
        "java.util.concurrent.ConcurrentHashMap".to_string(),
        "java.util.concurrent.atomic.AtomicLong".to_string(),
    ];

    let mut functions: Vec<Function> = namespace
        .functions
        .iter()
        .map(convert_function)
        .collect::<Result<Vec<_>>>()?;

    // Generate method bodies for functions against the generated wrapper class name.
    let class_name = namespace.name.clone();
    for func in &mut functions {
        func.body = body_gen::generate_function_body(func, &class_name);
    }

    let type_definitions: Vec<TypeDefinition> = namespace
        .type_definitions
        .iter()
        .filter_map(|td| convert_type_definition(td, &class_name).transpose())
        .collect::<Result<Vec<_>>>()?;

    let ffi_definitions: Vec<FfiDefinition> = namespace
        .ffi_definitions
        .iter()
        .filter_map(|ffi| convert_ffi_definition(ffi, &package_name, &namespace.name).transpose())
        .collect::<Result<Vec<_>>>()?;

    let async_fn_count = functions.iter().filter(|f| f.is_async).count();

    let has_callback_interface = type_definitions.iter().any(|td| {
        matches!(td, TypeDefinition::CallbackInterface(_))
    });

    Ok(Module {
        package_name,
        cdylib_name,
        name: namespace.name.clone(),
        crate_name: namespace.crate_name.clone(),
        docstring: namespace.docstring.clone().map(|d| d.to_string()),
        imports,
        functions,
        type_definitions,
        ffi_definitions,
        async_fn_count,
        has_callback_interface,
    })
}

fn convert_function(func: &general::Function) -> Result<Function> {
    let java_name = to_lower_camel_case(&func.name);
    Ok(Function {
        name: func.name.clone(),
        java_name,
        is_async: func.is_async,
        arguments: func
            .inputs
            .iter()
            .map(convert_argument)
            .collect::<Result<Vec<_>>>()?,
        return_type: func.return_type.as_ref().map(convert_type),
        throws: func.throws.as_ref().map(convert_type),
        docstring: func.docstring.clone(),
        ffi_func: RustFfiFunctionName(func.callable.ffi_func.0.clone()),
        checksum: func.checksum,
        body: String::new(), // filled in later by convert_namespace
    })
}

fn convert_argument(arg: &general::Argument) -> Result<Argument> {
    let java_name = escape_reserved_word(&to_lower_camel_case(&arg.name));
    let ty = convert_type_node(&arg.ty);
    // We create a temporary Argument to compute lower_code and native_expr
    let temp = Argument {
        name: arg.name.clone(),
        java_name: java_name.clone(),
        ty: ty.clone(),
        optional: arg.optional,
        default: arg.default.as_ref().map(convert_default),
        lower_code: String::new(),
        native_expr: String::new(),
    };
    let lower_code = body_gen::lower_code_for_arg(&temp);
    let native_expr = body_gen::native_expr_for_arg(&temp);
    Ok(Argument {
        name: arg.name.clone(),
        java_name,
        ty,
        optional: arg.optional,
        default: arg.default.as_ref().map(convert_default),
        lower_code,
        native_expr,
    })
}

/// Convert a general::TypeNode (wraps general::Type) to our TypeNode.
fn convert_type_node(ty_node: &general::TypeNode) -> TypeNode {
    convert_type(&ty_node.ty)
}

/// Convert a general::FfiTypeNode (wraps general::FfiType) to our FfiType.
fn convert_ffi_type_node(ffi_node: &general::FfiTypeNode) -> FfiType {
    convert_ffi_type(&ffi_node.ty)
}

fn convert_type(ty: &general::Type) -> TypeNode {
    match ty {
        general::Type::Int8 => TypeNode::Int8,
        general::Type::UInt8 => TypeNode::UInt8,
        general::Type::Int16 => TypeNode::Int16,
        general::Type::UInt16 => TypeNode::UInt16,
        general::Type::Int32 => TypeNode::Int32,
        general::Type::UInt32 => TypeNode::UInt32,
        general::Type::Int64 => TypeNode::Int64,
        general::Type::UInt64 => TypeNode::UInt64,
        general::Type::Float32 => TypeNode::Float32,
        general::Type::Float64 => TypeNode::Float64,
        general::Type::Boolean => TypeNode::Boolean,
        general::Type::String => TypeNode::String,
        general::Type::Bytes => TypeNode::Bytes,
        general::Type::Timestamp => TypeNode::Timestamp,
        general::Type::Duration => TypeNode::Duration,
        general::Type::Interface { namespace, name, .. } => TypeNode::Object {
            namespace: namespace.clone(),
            name: name.clone(),
        },
        general::Type::CallbackInterface { namespace, name } => TypeNode::CallbackInterface {
            namespace: namespace.clone(),
            name: name.clone(),
        },
        general::Type::Record { namespace, name } => TypeNode::Record {
            namespace: namespace.clone(),
            name: name.clone(),
        },
        general::Type::Enum { namespace, name, .. } => TypeNode::Enum {
            namespace: namespace.clone(),
            name: name.clone(),
        },
        general::Type::Optional { inner_type } => {
            TypeNode::Optional(Box::new(convert_type(inner_type)))
        }
        general::Type::Sequence { inner_type } => {
            TypeNode::Sequence(Box::new(convert_type(inner_type)))
        }
        general::Type::Map { value_type, .. } => {
            TypeNode::Map(Box::new(convert_type(value_type)))
        }
        general::Type::Custom { namespace, name, builtin } => TypeNode::Custom {
            namespace: namespace.clone(),
            name: name.clone(),
            builtin: Box::new(convert_type(builtin)),
        },
    }
}

fn convert_default(default: &general::DefaultValue) -> DefaultValue {
    match default {
        general::DefaultValue::Default(_ty) => DefaultValue::Default,
        general::DefaultValue::Literal(lit_node) => DefaultValue::Literal(convert_literal(&lit_node.lit)),
    }
}

fn convert_literal(_lit: &general::Literal) -> Literal {
    // TODO: implement literal conversion
    Literal::None
}

fn convert_type_definition(td: &general::TypeDefinition, class_name: &str) -> Result<Option<TypeDefinition>> {
    match td {
        general::TypeDefinition::Interface(int) => {
            let imp = match int.imp {
                general::ObjectImpl::Struct => ObjectImpl::Struct,
                general::ObjectImpl::Trait => ObjectImpl::Trait,
                general::ObjectImpl::CallbackTrait => ObjectImpl::CallbackTrait,
            };
            let obj_java_name = to_upper_camel_case(&int.name);
            let mut ctors: Vec<Constructor> = int
                .constructors
                .iter()
                .map(|c| {
                    Ok(Constructor {
                        name: c.name.clone(),
                        java_name: format!("new{}", to_upper_camel_case(&c.name)),
                        arguments: c
                            .inputs
                            .iter()
                            .map(convert_argument)
                            .collect::<Result<Vec<_>>>()?,
                        throws: c.throws.as_ref().map(convert_type),
                        ffi_func: RustFfiFunctionName(c.callable.ffi_func.0.clone()),
                        checksum: c.checksum,
                        body: String::new(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            for ctor in &mut ctors {
                ctor.body = body_gen::generate_constructor_body(ctor, class_name, &obj_java_name);
            }

            let mut methods: Vec<Method> = int
                .methods
                .iter()
                .map(|m| {
                    Ok(Method {
                        name: m.name.clone(),
                        java_name: to_lower_camel_case(&m.name),
                        is_async: m.is_async,
                        arguments: m
                            .inputs
                            .iter()
                            .map(convert_argument)
                            .collect::<Result<Vec<_>>>()?,
                        return_type: m.return_type.as_ref().map(convert_type),
                        throws: m.throws.as_ref().map(convert_type),
                        ffi_func: RustFfiFunctionName(m.callable.ffi_func.0.clone()),
                        checksum: m.checksum,
                        body: String::new(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            for method in &mut methods {
                method.body = body_gen::generate_method_body(method, class_name);
            }

            Ok(Some(TypeDefinition::Object(Object {
                name: int.name.clone(),
                java_name: obj_java_name,
                imp,
                docstring: int.docstring.as_ref().map(|d| d.to_string()),
                constructors: ctors,
                methods,
                ffi_func_clone: RustFfiFunctionName(int.ffi_func_clone.0.clone()),
                ffi_func_free: RustFfiFunctionName(int.ffi_func_free.0.clone()),
            })))
        }
        general::TypeDefinition::Record(rec) => {
            let java_name = to_upper_camel_case(&rec.name);
            let fields: Vec<Field> = rec
                .fields
                .iter()
                .map(|f| {
                    Ok(Field {
                        name: f.name.clone(),
                        java_name: to_lower_camel_case(&f.name),
                        ty: convert_type_node(&f.ty),
                        default: f.default.as_ref().map(convert_default),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let temp_record = Record {
                name: rec.name.clone(),
                java_name: java_name.clone(),
                docstring: rec.docstring.as_ref().map(|d| d.to_string()),
                fields: fields.clone(),
                write_body: String::new(),
                read_body: String::new(),
            };
            let write_body = body_gen::generate_record_write_body(&temp_record);
            let read_body = body_gen::generate_record_read_body(&temp_record);
            Ok(Some(TypeDefinition::Record(Record {
                name: rec.name.clone(),
                java_name,
                docstring: rec.docstring.as_ref().map(|d| d.to_string()),
                fields,
                write_body,
                read_body,
            })))
        }
        general::TypeDefinition::Enum(e) => {
            let java_name = to_upper_camel_case(&e.name);
            let variants: Vec<Variant> = e
                .variants
                .iter()
                .map(|v| {
                    Ok(Variant {
                        name: v.name.clone(),
                        java_name: to_upper_camel_case(&v.name),
                        fields: v
                            .fields
                            .iter()
                            .map(|f| {
                                Ok(Field {
                                    name: f.name.clone(),
                                    java_name: to_lower_camel_case(&f.name),
                                    ty: convert_type_node(&f.ty),
                                    default: f.default.as_ref().map(convert_default),
                                })
                            })
                            .collect::<Result<Vec<_>>>()?,
                        has_fields: !v.fields.is_empty(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let temp_enum = Enum {
                name: e.name.clone(),
                java_name: java_name.clone(),
                docstring: e.docstring.as_ref().map(|d| d.to_string()),
                variants: variants.clone(),
                is_flat: e.is_flat,
                write_body: String::new(),
                read_body: String::new(),
            };
            let write_body = body_gen::generate_enum_write_body(&temp_enum);
            let read_body = body_gen::generate_enum_read_body(&temp_enum);
            Ok(Some(TypeDefinition::Enum(Enum {
                name: e.name.clone(),
                java_name,
                docstring: e.docstring.as_ref().map(|d| d.to_string()),
                variants,
                is_flat: e.is_flat,
                write_body,
                read_body,
            })))
        }
        general::TypeDefinition::CallbackInterface(cbi) => {
            Ok(Some(TypeDefinition::CallbackInterface(CallbackInterface {
                name: cbi.name.clone(),
                java_name: to_upper_camel_case(&cbi.name),
                docstring: cbi.docstring.as_ref().map(|d| d.to_string()),
                methods: cbi
                    .methods
                    .iter()
                    .map(|m| {
                        Ok(Method {
                            name: m.name.clone(),
                            java_name: to_lower_camel_case(&m.name),
                            is_async: m.is_async,
                            arguments: m
                                .inputs
                                .iter()
                                .map(convert_argument)
                                .collect::<Result<Vec<_>>>()?,
                            return_type: m.return_type.as_ref().map(convert_type),
                            throws: m.throws.as_ref().map(convert_type),
                            ffi_func: RustFfiFunctionName(m.callable.ffi_func.0.clone()),
                            checksum: m.checksum,
                            body: String::new(), // callback methods are abstract in the interface
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                ffi_init_callback: RustFfiFunctionName(cbi.vtable.init_fn.0.clone()),
                vtable: None, // TODO: generate VTable
            })))
        }
        general::TypeDefinition::Simple(_ty_node) => {
            // Simple type aliases (e.g., Timestamp, Duration) don't need
            // their own Java type definitions — they are inlined where used.
            Ok(None)
        }
        general::TypeDefinition::Optional(_) => {
            Ok(None)
        }
        general::TypeDefinition::Sequence(_) => {
            Ok(None)
        }
        general::TypeDefinition::Map(_) => {
            Ok(None)
        }
        general::TypeDefinition::Custom(_) => {
            Ok(None)
        }
        general::TypeDefinition::External(_) => {
            Ok(None)
        }
    }
}

fn convert_ffi_definition(
    ffi: &general::FfiDefinition,
    _package_name: &str,
    _namespace: &str,
) -> Result<Option<FfiDefinition>> {
    match ffi {
        general::FfiDefinition::FunctionType(ft) => {
            Ok(Some(FfiDefinition::CallbackFunction(FfiCallbackFunction {
                name: ft.name.0.clone(),
                arguments: ft
                    .arguments
                    .iter()
                    .map(|a| {
                        Ok(FfiArgument {
                            name: a.name.clone(),
                            ty: convert_ffi_type_node(&a.ty),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                return_type: ft.return_type.ty.as_ref().map(convert_ffi_type_node),
                has_rust_call_status_arg: ft.has_rust_call_status_arg,
            })))
        }
        general::FfiDefinition::Struct(_st) => {
            // FfiStruct definitions (e.g., VTable structs) are handled
            // by the callback interface code generation; skip here.
            Ok(None)
        }
        general::FfiDefinition::RustFunction(func) => {
            // JNI naming: Java_<package>_<ClassName>_<method>
            // Class name is the namespace name (not upper-camel-cased).
            // Method name must have underscores escaped as _1 per JNI spec.
            let jni_method = func.name.0.replace('_', "_1");
            let jni_name = format!(
                "Java_{}_{}_{}",
                _package_name.replace('.', "_"),
                _namespace,
                jni_method
            );
            Ok(Some(FfiDefinition::RustFunction(FfiFunction {
                name: func.name.0.clone(),
                jni_name,
                arguments: func
                    .arguments
                    .iter()
                    .map(|a| {
                        Ok(FfiArgument {
                            name: a.name.clone(),
                            ty: convert_ffi_type_node(&a.ty),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                return_type: func.return_type.ty.as_ref().map(convert_ffi_type_node),
                has_rust_call_status_arg: func.has_rust_call_status_arg,
            })))
        }
    }
}

fn convert_ffi_type(ffi: &general::FfiType) -> FfiType {
    match ffi {
        general::FfiType::Int8 => FfiType::Int8,
        general::FfiType::UInt8 => FfiType::UInt8,
        general::FfiType::Int16 => FfiType::Int16,
        general::FfiType::UInt16 => FfiType::UInt16,
        general::FfiType::Int32 => FfiType::Int32,
        general::FfiType::UInt32 => FfiType::UInt32,
        general::FfiType::Int64 => FfiType::Int64,
        general::FfiType::UInt64 => FfiType::UInt64,
        general::FfiType::Float32 => FfiType::Float32,
        general::FfiType::Float64 => FfiType::Float64,
        general::FfiType::RustBuffer(_) => FfiType::RustBuffer,
        general::FfiType::ForeignBytes => FfiType::ForeignBytes,
        general::FfiType::Function(name) => FfiType::Function(name.0.clone()),
        general::FfiType::Struct(name) => FfiType::Struct(name.0.clone()),
        general::FfiType::Handle(_) => FfiType::Handle,
        general::FfiType::RustCallStatus => FfiType::VoidPointer,
        general::FfiType::Reference(inner) => FfiType::Reference(Box::new(convert_ffi_type(inner))),
        general::FfiType::MutReference(inner) => FfiType::Reference(Box::new(convert_ffi_type(inner))),
        general::FfiType::VoidPointer => FfiType::VoidPointer,
    }
}

// --- Naming helpers ---

pub fn to_upper_camel_case(name: &str) -> String {
    heck::ToUpperCamelCase::to_upper_camel_case(name)
}

pub fn to_lower_camel_case(name: &str) -> String {
    heck::ToLowerCamelCase::to_lower_camel_case(name)
}

pub fn to_snake_case(name: &str) -> String {
    heck::ToSnakeCase::to_snake_case(name)
}

/// Escape Java reserved words by prefixing with underscore.
fn escape_reserved_word(name: &str) -> String {
    const RESERVED: &[&str] = &[
        "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char",
        "class", "const", "continue", "default", "do", "double", "else", "enum",
        "extends", "false", "final", "finally", "float", "for", "goto", "if",
        "implements", "import", "instanceof", "int", "interface", "long", "native",
        "new", "null", "package", "private", "protected", "public", "return",
        "short", "static", "strictfp", "super", "switch", "synchronized", "this",
        "throw", "throws", "transient", "true", "try", "void", "volatile", "while",
        "_", "var", "yield", "record", "sealed", "permits",
    ];
    if RESERVED.contains(&name) {
        format!("_{name}")
    } else {
        name.to_string()
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use uniffi_bindgen::pipeline::general::{FfiType as GeneralFfiType, HandleKind, Type as GeneralType};

    // ========== convert_type tests ==========

    #[test]
    fn convert_primitive_types() {
        assert!(matches!(convert_type(&GeneralType::Int8), TypeNode::Int8));
        assert!(matches!(convert_type(&GeneralType::UInt8), TypeNode::UInt8));
        assert!(matches!(convert_type(&GeneralType::Int16), TypeNode::Int16));
        assert!(matches!(convert_type(&GeneralType::Int32), TypeNode::Int32));
        assert!(matches!(convert_type(&GeneralType::Int64), TypeNode::Int64));
        assert!(matches!(convert_type(&GeneralType::Float32), TypeNode::Float32));
        assert!(matches!(convert_type(&GeneralType::Float64), TypeNode::Float64));
        assert!(matches!(convert_type(&GeneralType::Boolean), TypeNode::Boolean));
        assert!(matches!(convert_type(&GeneralType::String), TypeNode::String));
        assert!(matches!(convert_type(&GeneralType::Bytes), TypeNode::Bytes));
        assert!(matches!(convert_type(&GeneralType::Timestamp), TypeNode::Timestamp));
        assert!(matches!(convert_type(&GeneralType::Duration), TypeNode::Duration));
    }

    #[test]
    fn convert_interface_to_object() {
        let result = convert_type(&GeneralType::Interface {
            namespace: "test_ns".into(),
            name: "MyType".into(),
            imp: general::ObjectImpl::Struct,
        });
        assert!(matches!(result, TypeNode::Object { namespace, name }
            if namespace == "test_ns" && name == "MyType"));
    }

    #[test]
    fn convert_record_type() {
        let result = convert_type(&GeneralType::Record {
            namespace: "ns".into(),
            name: "MyRecord".into(),
        });
        assert!(matches!(result, TypeNode::Record { namespace, name }
            if namespace == "ns" && name == "MyRecord"));
    }

    #[test]
    fn convert_enum_type() {
        let result = convert_type(&GeneralType::Enum {
            namespace: "ns".into(),
            name: "MyEnum".into(),
        });
        assert!(matches!(result, TypeNode::Enum { namespace, name }
            if namespace == "ns" && name == "MyEnum"));
    }

    #[test]
    fn convert_optional_type() {
        let result = convert_type(&GeneralType::Optional {
            inner_type: Box::new(GeneralType::Int32),
        });
        assert!(matches!(result, TypeNode::Optional(inner)
            if matches!(*inner, TypeNode::Int32)));
    }

    #[test]
    fn convert_sequence_type() {
        let result = convert_type(&GeneralType::Sequence {
            inner_type: Box::new(GeneralType::String),
        });
        assert!(matches!(result, TypeNode::Sequence(inner)
            if matches!(*inner, TypeNode::String)));
    }

    // ========== convert_ffi_type tests ==========

    #[test]
    fn convert_ffi_primitives() {
        assert!(matches!(convert_ffi_type(&GeneralFfiType::Int8), FfiType::Int8));
        assert!(matches!(convert_ffi_type(&GeneralFfiType::UInt8), FfiType::UInt8));
        assert!(matches!(convert_ffi_type(&GeneralFfiType::Int16), FfiType::Int16));
        assert!(matches!(convert_ffi_type(&GeneralFfiType::Int32), FfiType::Int32));
        assert!(matches!(convert_ffi_type(&GeneralFfiType::Int64), FfiType::Int64));
        assert!(matches!(convert_ffi_type(&GeneralFfiType::Float32), FfiType::Float32));
        assert!(matches!(convert_ffi_type(&GeneralFfiType::Float64), FfiType::Float64));
    }

    #[test]
    fn convert_ffi_rust_buffer() {
        assert!(matches!(
            convert_ffi_type(&GeneralFfiType::RustBuffer(None)),
            FfiType::RustBuffer
        ));
    }

    #[test]
    fn convert_ffi_handle() {
        assert!(matches!(
            convert_ffi_type(&GeneralFfiType::Handle(HandleKind::RustFuture)),
            FfiType::Handle
        ));
    }

    #[test]
    fn convert_ffi_void_pointer() {
        assert!(matches!(convert_ffi_type(&GeneralFfiType::VoidPointer), FfiType::VoidPointer));
    }

    // ========== naming helpers tests ==========

    #[test]
    fn upper_camel_case() {
        assert_eq!(to_upper_camel_case("hello_world"), "HelloWorld");
        assert_eq!(to_upper_camel_case("hello-world"), "HelloWorld");
        assert_eq!(to_upper_camel_case("foo"), "Foo");
    }

    #[test]
    fn lower_camel_case() {
        assert_eq!(to_lower_camel_case("HelloWorld"), "helloWorld");
        assert_eq!(to_lower_camel_case("hello_world"), "helloWorld");
    }

    #[test]
    fn snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
    }

    #[test]
    fn escape_reserved_word_test() {
        assert_eq!(escape_reserved_word("class"), "_class");
        assert_eq!(escape_reserved_word("myVar"), "myVar");
        assert_eq!(escape_reserved_word("int"), "_int");
    }
}
