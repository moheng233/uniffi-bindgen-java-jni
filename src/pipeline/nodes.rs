// Java JNI Intermediate Representation nodes.
//
// These nodes define the Java-specific IR used for code generation.
// We inherit many nodes from the general IR using `use_prev_node!`,
// and define Java-specific nodes where needed.

use askama::Template;

// Import filters so that askama-generated code can find them via `filters::filter_name`
use super::filters;

/// The root node of the Java IR.
#[derive(Debug, Clone)]
pub struct Root {
    /// In library mode, the library path the user passed to us
    pub cdylib: Option<String>,
    /// Java modules, keyed by namespace name
    pub modules: indexmap::IndexMap<String, Module>,
}

/// A Java module represents one namespace (crate) and maps to
/// a Java package with multiple source files.
#[derive(Debug, Clone, Template)]
#[template(syntax = "java", escape = "none", path = "java/wrapper.java")]
pub struct Module {
    /// Java package name
    pub package_name: String,
    /// cdylib library name for System.loadLibrary
    pub cdylib_name: String,
    /// The namespace name (used for class naming)
    pub name: String,
    /// The crate name
    pub crate_name: String,
    /// Docstring for the namespace
    pub docstring: Option<String>,
    /// Java imports needed by this module
    pub imports: Vec<String>,
    /// Top-level functions
    pub functions: Vec<Function>,
    /// Type definitions (objects, enums, records, callback interfaces)
    pub type_definitions: Vec<TypeDefinition>,
    /// FFI definitions (native method declarations)
    pub ffi_definitions: Vec<FfiDefinition>,
    /// Number of async functions
    pub async_fn_count: usize,
    /// Whether this module has callback interfaces
    pub has_callback_interface: bool,
}

/// A top-level function in the Java bindings.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub java_name: String,
    pub is_async: bool,
    pub arguments: Vec<Argument>,
    pub return_type: Option<TypeNode>,
    pub throws: Option<TypeNode>,
    pub docstring: Option<String>,
    pub ffi_func: RustFfiFunctionName,
    pub checksum: Option<u16>,
}

/// Function/method argument.
#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub java_name: String,
    pub ty: TypeNode,
    pub optional: bool,
    pub default: Option<DefaultValue>,
}

/// A type definition in the Java bindings.
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Object(Object),
    Record(Record),
    Enum(Enum),
    CallbackInterface(CallbackInterface),
}

/// An Object (Rust struct/trait implemented in Rust or Java).
#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub java_name: String,
    pub imp: ObjectImpl,
    pub docstring: Option<String>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub ffi_func_clone: RustFfiFunctionName,
    pub ffi_func_free: RustFfiFunctionName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectImpl {
    Struct,
    Trait,
    CallbackTrait,
}

/// A constructor for an Object.
#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub java_name: String,
    pub arguments: Vec<Argument>,
    pub throws: Option<TypeNode>,
    pub ffi_func: RustFfiFunctionName,
    pub checksum: Option<u16>,
}

/// A method on an Object, Record, or Enum.
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub java_name: String,
    pub is_async: bool,
    pub arguments: Vec<Argument>,
    pub return_type: Option<TypeNode>,
    pub throws: Option<TypeNode>,
    pub ffi_func: RustFfiFunctionName,
    pub checksum: Option<u16>,
}

/// A Record (data struct).
#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub java_name: String,
    pub docstring: Option<String>,
    pub fields: Vec<Field>,
}

/// A field in a Record or Enum variant.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub java_name: String,
    pub ty: TypeNode,
    pub default: Option<DefaultValue>,
}

/// An Enum type.
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub java_name: String,
    pub docstring: Option<String>,
    pub variants: Vec<Variant>,
    pub is_flat: bool,
}

/// An Enum variant.
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub java_name: String,
    pub fields: Vec<Field>,
    pub has_fields: bool,
}

/// A Callback Interface (trait that Java implements, Rust calls).
#[derive(Debug, Clone)]
pub struct CallbackInterface {
    pub name: String,
    pub java_name: String,
    pub docstring: Option<String>,
    pub methods: Vec<Method>,
    pub ffi_init_callback: RustFfiFunctionName,
    pub vtable: Option<VTable>,
}

/// VTable for callback interface.
#[derive(Debug, Clone)]
pub struct VTable {
    pub methods: Vec<VTableMethod>,
}

#[derive(Debug, Clone)]
pub struct VTableMethod {
    pub ffi_callback: FfiCallbackFunction,
    pub method: Method,
}

/// A JNI native method declaration or FFI function name.
#[derive(Debug, Clone)]
pub struct RustFfiFunctionName(pub String);

impl RustFfiFunctionName {
    /// Return the function name string.
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// An FFI callback function definition.
#[derive(Debug, Clone)]
pub struct FfiCallbackFunction {
    pub name: String,
    pub arguments: Vec<FfiArgument>,
    pub return_type: Option<FfiType>,
    pub has_rust_call_status_arg: bool,
}

/// An FFI argument.
#[derive(Debug, Clone)]
pub struct FfiArgument {
    pub name: String,
    pub ty: FfiType,
}

/// An FFI type.
#[derive(Debug, Clone)]
pub enum FfiType {
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    Boolean,
    String,
    Bytes,
    Handle,
    RustBuffer,
    RustArc,
    VoidPointer,
    Function(String),
    Struct(String),
    Callback(String),
    Reference(Box<FfiType>),
}

/// An FFI definition: either a callback type or a Rust function.
#[derive(Debug, Clone)]
pub enum FfiDefinition {
    CallbackFunction(FfiCallbackFunction),
    RustFunction(FfiFunction),
}

/// An FFI function (native method).
#[derive(Debug, Clone)]
pub struct FfiFunction {
    pub name: String,
    pub jni_name: String,
    pub arguments: Vec<FfiArgument>,
    pub return_type: Option<FfiType>,
    pub has_rust_call_status_arg: bool,
}

/// A type node representing a UniFFI type in Java context.
#[derive(Debug, Clone)]
pub enum TypeNode {
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    Boolean,
    String,
    Bytes,
    Timestamp,
    Duration,
    Object {
        namespace: String,
        name: String,
    },
    CallbackInterface {
        namespace: String,
        name: String,
    },
    Record {
        namespace: String,
        name: String,
    },
    Enum {
        namespace: String,
        name: String,
    },
    Optional(Box<TypeNode>),
    Sequence(Box<TypeNode>),
    Map(Box<TypeNode>),
    Custom {
        namespace: String,
        name: String,
        builtin: Box<TypeNode>,
    },
    External {
        namespace: String,
        name: String,
    },
}

/// A default value for an argument or field.
#[derive(Debug, Clone)]
pub enum DefaultValue {
    Literal(Literal),
    Default,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Boolean(bool),
    String(String),
    Int(i64, Radix, TypeNode),
    UInt(u64, Radix, TypeNode),
    Float(String, TypeNode),
    None,
    Some { inner: Box<DefaultValue> },
    EmptySequence,
    EmptyMap,
    Enum(String, Box<TypeNode>),
}

#[derive(Debug, Clone)]
pub enum Radix {
    Octal,
    Decimal,
    Hexadecimal,
}
