use camino::Utf8PathBuf;
use clap::Parser;
use std::collections::HashMap;
use uniffi_bindgen::{BindgenLoader, BindgenPaths};

use uniffi_bindgen_java_jni::pipeline::config::{self, JavaConfig};

/// Java JNI bindings generator for UniFFI.
///
/// Generates both Java source files (with native method declarations)
/// and a Rust JNI glue crate that bridges JNI calls to UniFFI FFI functions.
#[derive(Parser)]
#[command(name = "uniffi-bindgen-java-jni")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the cdylib file (library mode) or UDL file
    #[arg(long, short)]
    source: Utf8PathBuf,

    /// Directory to write generated Java files
    #[arg(long = "java-out-dir", default_value = "./generated/java")]
    java_out_dir: Utf8PathBuf,

    /// Directory to write generated Rust JNI glue crate
    #[arg(long = "rust-out-dir", default_value = "./generated/rust-glue")]
    rust_out_dir: Utf8PathBuf,

    /// Path to a global config file (uniffi.toml compatible)
    #[arg(long)]
    config: Option<Utf8PathBuf>,

    /// Limit generation to a single crate
    #[arg(long = "crate")]
    crate_filter: Option<String>,

    /// Exclude dependencies when running cargo metadata
    #[arg(long)]
    metadata_no_deps: bool,

    /// Path to the main crate (for Rust glue Cargo.toml dependency)
    #[arg(long = "main-crate-path")]
    main_crate_path: Option<Utf8PathBuf>,

    /// Override the main crate's package name in the generated Cargo.toml.
    /// If not provided, derived from crate metadata.
    #[arg(long = "main-crate-name")]
    main_crate_name: Option<String>,

    /// Override a dependency in the generated Cargo.toml.
    /// Format: name=spec. Repeatable.
    /// E.g. -d 'uniffi={ git = "https://...", tag = "v0.31.1" }'
    #[arg(short = 'd', long = "dependency", value_parser = parse_dep_override)]
    dependency_overrides: Vec<(String, String)>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup paths
    let mut paths = BindgenPaths::default();

    // Load and parse Java config
    let java_config = if let Some(ref config_path) = cli.config {
        // Add as a config override layer so load_pipeline_initial_root can see it
        paths.add_config_override_layer(config_path.clone());

        let contents = fs_err::read_to_string(config_path)?;
        let root_toml: toml::Value = toml::from_str(&contents)?;
        config::parse_java_config(&root_toml)?
    } else {
        JavaConfig::default()
    };

    // Create loader
    let loader = BindgenLoader::new(paths);

    // Build dependency overrides map
    let dep_overrides: HashMap<String, String> = cli.dependency_overrides.into_iter().collect();

    // Generate bindings
    uniffi_bindgen_java_jni::generate_java_jni_bindings(
        &loader,
        &cli.source,
        &cli.java_out_dir,
        &cli.rust_out_dir,
        cli.crate_filter.as_deref(),
        &java_config,
        cli.main_crate_path.as_deref(),
        cli.main_crate_name.as_deref(),
        &dep_overrides,
    )?;

    println!("Java JNI bindings generated successfully!");
    println!("  Java output: {}", cli.java_out_dir);
    println!("  Rust output: {}", cli.rust_out_dir);

    Ok(())
}

/// Parse a dependency override argument of the form `name=spec`.
/// Splits on the first `=` only, since the spec may contain `=` (e.g. TOML inline tables).
fn parse_dep_override(s: &str) -> Result<(String, String), String> {
    let (name, spec) = s.split_once('=')
        .ok_or_else(|| format!("expected format name=spec, got: {s}"))?;
    Ok((name.to_string(), spec.to_string()))
}
