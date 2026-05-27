//! Cargo.toml generation for the Rust JNI glue crate.

use askama::Template;
use camino::Utf8Path;
use std::collections::HashMap;
use crate::pipeline::nodes::Root;

/// A dependency entry for the generated Cargo.toml.
#[derive(Debug, Clone)]
struct Dependency {
    name: String,
    /// TOML value part, e.g. `"0.31"` or `{ git = "...", tag = "..." }`
    spec: String,
}

/// Template for rendering the glue crate's Cargo.toml.
#[derive(Template)]
#[template(escape = "none", path = "rust/cargo_toml.rs")]
struct CargoTomlTemplate<'a> {
    crate_name: &'a str,
    /// Name of the main crate package (for the dependency key).
    main_crate_name: &'a str,
    /// Path to the main crate, relative to the glue crate directory.
    main_crate_path: Option<&'a str>,
    /// Whether callback interfaces exist.
    #[allow(dead_code)]
    has_callbacks: bool,
    /// All dependencies (including defaults + overrides).
    dependencies: &'a [Dependency],
}

/// Generate the Cargo.toml content for the glue crate.
///
/// Default dependencies (jni, uniffi, once_cell) are built first,
/// then `dependency_overrides` replaces matching keys or adds new entries.
pub fn generate_cargo_toml(
    crate_name: &str,
    root: &Root,
    main_crate_path: Option<&Utf8Path>,
    main_crate_name: Option<&str>,
    dependency_overrides: &HashMap<String, String>,
) -> String {
    let main_crate_name = main_crate_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            root.modules
                .values()
                .next()
                .map(|m| m.crate_name.replace('_', "-"))
                .unwrap_or_else(|| "main-crate".to_string())
        });

    let has_callbacks = root.modules.values().any(|m| m.has_callback_interface);

    // Build default dependency map (order via IndexMap to keep jni before uniffi)
    let mut deps: indexmap::IndexMap<String, String> = indexmap::IndexMap::from([
        ("jni".to_string(), r#""0.21""#.to_string()),
        ("uniffi".to_string(), r#""0.31""#.to_string()),
    ]);
    if has_callbacks {
        deps.insert("once_cell".to_string(), r#""1.20""#.to_string());
    }
    // Apply overrides: replace matching keys, add new ones
    for (name, spec) in dependency_overrides {
        deps.insert(name.clone(), spec.clone());
    }

    let dependencies: Vec<Dependency> = deps
        .into_iter()
        .map(|(name, spec)| Dependency { name, spec })
        .collect();

    let tmpl = CargoTomlTemplate {
        crate_name,
        main_crate_name: &main_crate_name,
        main_crate_path: main_crate_path.map(|p| p.as_str()),
        has_callbacks,
        dependencies: &dependencies,
    };
    tmpl.render().expect("Failed to render Cargo.toml template")
}
