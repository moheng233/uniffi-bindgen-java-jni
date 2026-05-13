use anyhow::Result;
use uniffi_bindgen::pipeline::general;

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
    ];

    let functions: Vec<Function> = namespace
        .functions
        .iter()
        .map(convert_function)
        .collect::<Result<Vec<_>>>()?;

    let type_definitions: Vec<TypeDefinition> = namespace
        .type_definitions
        .iter()
        .map(convert_type_definition)
        .collect::<Result<Vec<_>>>()?;

    let ffi_definitions: Vec<FfiDefinition> = namespace
        .ffi_definitions
        .iter()
        .map(|ffi| convert_ffi_definition(ffi, &package_name, &namespace.name))
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
    })
}

fn convert_argument(arg: &general::Argument) -> Result<Argument> {
    let java_name = escape_reserved_word(&to_lower_camel_case(&arg.name));
    Ok(Argument {
        name: arg.name.clone(),
        java_name,
        ty: convert_type_node(&arg.ty),
        optional: arg.optional,
        default: arg.default.as_ref().map(convert_default),
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

fn convert_type_definition(td: &general::TypeDefinition) -> Result<TypeDefinition> {
    match td {
        general::TypeDefinition::Interface(int) => {
            let imp = match int.imp {
                general::ObjectImpl::Struct => ObjectImpl::Struct,
                general::ObjectImpl::Trait => ObjectImpl::Trait,
                general::ObjectImpl::CallbackTrait => ObjectImpl::CallbackTrait,
            };
            Ok(TypeDefinition::Object(Object {
                name: int.name.clone(),
                java_name: to_upper_camel_case(&int.name),
                imp,
                docstring: int.docstring.as_ref().map(|d| d.to_string()),
                constructors: int
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
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                methods: int
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
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                ffi_func_clone: RustFfiFunctionName(int.ffi_func_clone.0.clone()),
                ffi_func_free: RustFfiFunctionName(int.ffi_func_free.0.clone()),
            }))
        }
        general::TypeDefinition::Record(rec) => Ok(TypeDefinition::Record(Record {
            name: rec.name.clone(),
            java_name: to_upper_camel_case(&rec.name),
            docstring: rec.docstring.as_ref().map(|d| d.to_string()),
            fields: rec
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
        })),
        general::TypeDefinition::Enum(e) => Ok(TypeDefinition::Enum(Enum {
            name: e.name.clone(),
            java_name: to_upper_camel_case(&e.name),
            docstring: e.docstring.as_ref().map(|d| d.to_string()),
            variants: e
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
                .collect::<Result<Vec<_>>>()?,
            is_flat: e.is_flat,
        })),
        general::TypeDefinition::CallbackInterface(cbi) => {
            Ok(TypeDefinition::CallbackInterface(CallbackInterface {
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
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                ffi_init_callback: RustFfiFunctionName(cbi.vtable.init_fn.0.clone()),
                vtable: None, // TODO: generate VTable
            }))
        }
        general::TypeDefinition::Simple(_ty_node) => {
            // Simple types don't generate Java type definitions
            Err(anyhow::anyhow!("Simple type definition not expected here"))
        }
        general::TypeDefinition::Optional(_) => {
            // Compound types don't generate Java type definitions
            Err(anyhow::anyhow!("Optional type definition not expected here"))
        }
        general::TypeDefinition::Sequence(_) => {
            Err(anyhow::anyhow!("Sequence type definition not expected here"))
        }
        general::TypeDefinition::Map(_) => {
            Err(anyhow::anyhow!("Map type definition not expected here"))
        }
        general::TypeDefinition::Custom(_) => {
            // Custom types are handled by the FfiConverter
            Err(anyhow::anyhow!("Custom type definition not expected here"))
        }
        general::TypeDefinition::External(_) => {
            // External types are handled separately
            Err(anyhow::anyhow!("External type definition not expected here"))
        }
    }
}

fn convert_ffi_definition(
    ffi: &general::FfiDefinition,
    _package_name: &str,
    _namespace: &str,
) -> Result<FfiDefinition> {
    match ffi {
        general::FfiDefinition::FunctionType(ft) => {
            Ok(FfiDefinition::CallbackFunction(FfiCallbackFunction {
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
            }))
        }
        general::FfiDefinition::Struct(_st) => {
            // FfiStruct definitions (e.g., VTable structs) are handled
            // by the callback interface code generation
            Err(anyhow::anyhow!("FFI struct definition handled by callback codegen"))
        }
        general::FfiDefinition::RustFunction(func) => {
            let jni_name = format!(
                "Java_{}_{}_{}",
                _package_name.replace('.', "_"),
                to_upper_camel_case(_namespace),
                func.name.0
            );
            Ok(FfiDefinition::RustFunction(FfiFunction {
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
            }))
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
        general::FfiType::ForeignBytes => FfiType::VoidPointer,
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
