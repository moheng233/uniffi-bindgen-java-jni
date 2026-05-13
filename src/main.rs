use camino::Utf8PathBuf;
use clap::Parser;
use uniffi_bindgen::{BindgenLoader, BindgenPaths};

use uniffi_bindgen_java_jna::pipeline::config::{self, JavaConfig};

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

    // Generate bindings
    uniffi_bindgen_java_jna::generate_java_jni_bindings(
        &loader,
        &cli.source,
        &cli.java_out_dir,
        &cli.rust_out_dir,
        cli.crate_filter.as_deref(),
        &java_config,
        cli.main_crate_path.as_deref(),
    )?;

    println!("Java JNI bindings generated successfully!");
    println!("  Java output: {}", cli.java_out_dir);
    println!("  Rust output: {}", cli.rust_out_dir);

    Ok(())
}

