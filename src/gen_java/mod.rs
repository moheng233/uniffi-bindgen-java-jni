//! Java code generation module.
//!
//! This module handles rendering the Java IR nodes into actual Java source files
//! using Askama templates.

use anyhow::Result;
use askama::Template;
use camino::Utf8Path;
use fs_err as fs;

use crate::pipeline::nodes::*;

/// Generate Java code from the Java IR root node.
pub fn generate_java_code(
    root: &Root,
    out_dir: &Utf8Path,
    crate_filter: Option<&str>,
) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    for module in root.modules.values() {
        if let Some(filter) = crate_filter
            && module.crate_name != filter {
                continue;
            }

        // Generate the wrapper file for this module
        let rendered = module.render().map_err(|e| {
            anyhow::anyhow!("Failed to render Java module {}: {}", module.name, e)
        })?;

        let package_dir = module.package_name.replace('.', "/");
        let dir = out_dir.join(&package_dir);
        fs::create_dir_all(&dir)?;

        let filename = format!("{}.java", module.name);
        let path = dir.join(filename);
        fs::write(&path, rendered)?;
        println!("Generated: {}", path);
    }

    Ok(())
}

/// Java naming oracle.
pub struct JavaCodeOracle;

impl JavaCodeOracle {
    pub fn class_name(name: &str) -> String {
        heck::ToUpperCamelCase::to_upper_camel_case(name)
    }

    pub fn fn_name(name: &str) -> String {
        let camel = heck::ToLowerCamelCase::to_lower_camel_case(name);
        escape_reserved(&camel)
    }

    pub fn var_name(name: &str) -> String {
        let camel = heck::ToLowerCamelCase::to_lower_camel_case(name);
        escape_reserved(&camel)
    }
}

fn escape_reserved(name: &str) -> String {
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
