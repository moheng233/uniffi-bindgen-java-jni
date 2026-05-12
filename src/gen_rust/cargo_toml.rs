//! Cargo.toml generation for the Rust JNI glue crate.

use crate::pipeline::nodes::Root;

/// Generate the Cargo.toml content for the glue crate.
pub fn generate_cargo_toml(crate_name: &str, _root: &Root) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
uniffi = {{ version = "0.31", features = ["builtin-bindgen"] }}
# TODO: Add dependency on the main crate
# {{main_crate}} = {{ path = "{{main_crate_path}}" }}
"#,
        crate_name
    )
}
