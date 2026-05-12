use std::collections::{HashMap, HashSet};

use super::config::JavaConfig;

/// Context for the Java JNI pipeline pass.
///
/// Holds the Java-specific configuration and tracks the current
/// namespace, crate, and type being processed.
#[derive(Default, Clone)]
pub struct Context {
    pub config: JavaConfig,
    pub current_namespace_name: Option<String>,
    pub current_crate_name: Option<String>,
    pub current_type: Option<String>,
    pub current_variant: Option<String>,
    pub current_arg_or_field_type: Option<String>,
    pub names_used_as_error: HashSet<String>,
    pub rename_tables: HashMap<String, HashMap<String, String>>,
    pub exclude_sets: HashMap<String, HashSet<String>>,
}

impl Context {
    pub fn new(config: JavaConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn namespace_name(&self) -> anyhow::Result<String> {
        self.current_namespace_name
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Context.current_namespace_name not set"))
    }

    pub fn crate_name(&self) -> anyhow::Result<String> {
        self.current_crate_name
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Context.crate_name not set"))
    }

    /// Update context from the root node.
    pub fn update_from_root(
        &mut self,
        _root: &super::general::Root,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Update context from a namespace node.
    pub fn update_from_namespace(
        &mut self,
        namespace: &super::general::Namespace,
    ) -> anyhow::Result<()> {
        self.current_namespace_name = Some(namespace.name.clone());
        Ok(())
    }

    pub fn package_name(&self) -> anyhow::Result<String> {
        let namespace = self.namespace_name()?;
        Ok(self.config.package_name(&namespace))
    }

    pub fn cdylib_name(&self) -> anyhow::Result<String> {
        let namespace = self.namespace_name()?;
        Ok(self.config.cdylib_name(&namespace))
    }
}
