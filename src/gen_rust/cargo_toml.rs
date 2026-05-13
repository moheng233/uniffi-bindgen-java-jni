//! Cargo.toml generation for the Rust JNI glue crate.

use askama::Template;
use camino::Utf8Path;
use crate::pipeline::nodes::Root;

/// Template for rendering the glue crate's Cargo.toml.
#[derive(Template)]
#[template(escape = "none", path = "rust/cargo_toml.rs")]
struct CargoTomlTemplate<'a> {
    crate_name: &'a str,
    /// Name of the main crate package (for the dependency key).
    main_crate_name: &'a str,
    /// Path to the main crate, relative to the glue crate directory.
    main_crate_path: Option<&'a str>,
}

/// Generate the Cargo.toml content for the glue crate.
pub fn generate_cargo_toml(
    crate_name: &str,
    root: &Root,
    main_crate_path: Option<&Utf8Path>,
) -> String {
    // Get the first module's crate_name as the main crate dependency name.
    // UniFFI normalizes names to underscores, but Cargo.toml package names
    // may use hyphens. Replace underscores with hyphens for compatibility.
    let main_crate_name = root
        .modules
        .values()
        .next()
        .map(|m| m.crate_name.replace('_', "-"))
        .unwrap_or_else(|| "main-crate".to_string());
    let tmpl = CargoTomlTemplate {
        crate_name,
        main_crate_name: &main_crate_name,
        main_crate_path: main_crate_path.map(|p| p.as_str()),
    };
    tmpl.render().expect("Failed to render Cargo.toml template")
}

