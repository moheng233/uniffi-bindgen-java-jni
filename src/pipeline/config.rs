/// Java JNI specific configuration parsed from uniffi.toml's [bindings.java] section.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct JavaConfig {
    /// Java package name for generated code (default: uniffi.{namespace})
    pub package_name: Option<String>,

    /// Name of the cdylib to load via System.loadLibrary()
    pub cdylib_name: Option<String>,

    /// Custom type mappings
    #[serde(default)]
    pub custom_types: indexmap::IndexMap<String, CustomTypeConfig>,
}

/// Configuration for a custom type mapping.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct CustomTypeConfig {
    /// Java imports needed for this type
    pub imports: Option<Vec<String>>,
    /// The fully qualified Java type name
    pub type_name: Option<String>,
    /// Expression to convert from the builtin FFI type to the custom type (lift)
    pub lift: String,
    /// Expression to convert from the custom type to the builtin FFI type (lower)
    pub lower: String,
}

impl CustomTypeConfig {
    pub fn lift(&self, name: &str) -> String {
        self.lift.replace("{}", name)
    }

    pub fn lower(&self, name: &str) -> String {
        self.lower.replace("{}", name)
    }
}

impl JavaConfig {
    /// Resolve the effective Java package name for a given namespace.
    pub fn package_name(&self, namespace: &str) -> String {
        self.package_name
            .clone()
            .unwrap_or_else(|| format!("uniffi.{}", namespace.replace('-', "_")))
    }

    /// Resolve the effective cdylib name.
    pub fn cdylib_name(&self, namespace: &str) -> String {
        self.cdylib_name
            .clone()
            .unwrap_or_else(|| format!("uniffi_{}", namespace.replace('-', "_")))
    }
}
