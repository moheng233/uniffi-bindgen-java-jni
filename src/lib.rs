use anyhow::Result;
use camino::Utf8Path;
use indexmap::IndexMap;
use uniffi_bindgen::BindgenLoader;

pub mod pipeline;
pub mod gen_java;
pub mod gen_rust;

use crate::pipeline::config::JavaConfig;
use crate::pipeline::modules;
use crate::pipeline::nodes;

/// Generate Java JNI bindings and Rust glue code.
///
/// Arguments:
/// - `loader`: Initialized BindgenLoader with metadata and paths
/// - `source`: Path to the cdylib or UDL file
/// - `java_out_dir`: Directory for generated Java files
/// - `rust_out_dir`: Directory for generated Rust JNI glue crate
/// - `crate_filter`: Optional crate name to filter
/// - `config`: Java-specific configuration (package name, cdylib name, custom types)
/// - `main_crate_path`: Optional path to the main crate for Rust glue Cargo.toml dependency
pub fn generate_java_jni_bindings(
    loader: &BindgenLoader,
    source: &Utf8Path,
    java_out_dir: &Utf8Path,
    rust_out_dir: &Utf8Path,
    crate_filter: Option<&str>,
    config: &JavaConfig,
    main_crate_path: Option<&Utf8Path>,
) -> Result<()> {
    // Phase 1: Load metadata and create the initial IR
    let metadata = loader.load_metadata(source)?;

    if let Some(crate_name) = crate_filter
        && !metadata.contains_key(crate_name) {
            anyhow::bail!("No UniFFI metadata found for crate {crate_name}");
        }

    let initial_root = loader.load_pipeline_initial_root(source, metadata)?;

    // Phase 2: Run the general pipeline to get general::Root
    let general_root = pipeline::general_pipeline().execute(initial_root)?;

    // Phase 3: Convert general::Root to Java IR
    let java_root = convert_to_java_root(&general_root, config)?;

    // Phase 4: Generate Java code
    crate::gen_java::generate_java_code(&java_root, java_out_dir, crate_filter)?;

    // Phase 5: Generate Rust JNI glue code
    crate::gen_rust::generate_rust_glue(&java_root, rust_out_dir, crate_filter, main_crate_path)?;

    Ok(())
}

/// Convert the general IR root to our Java-specific IR root.
fn convert_to_java_root(
    general_root: &uniffi_bindgen::pipeline::general::Root,
    config: &JavaConfig,
) -> Result<nodes::Root> {
    let cdylib = general_root.cdylib.clone();

    let mut modules = IndexMap::new();

    for (name, namespace) in &general_root.namespaces {
        let module = modules::convert_namespace(namespace, config, cdylib.as_deref())?;
        modules.insert(name.clone(), module);
    }

    Ok(nodes::Root { cdylib, modules })
}
