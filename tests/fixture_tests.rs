//! Integration tests using uniffi-rs official fixture crates.
//!
//! Fixture crates are pulled via `[dev-dependencies]` git references.
//! `uniffi_testing::UniFFITestHelper` handles building fixtures and finding cdylib paths.
//!
//! Pattern adapted from IronCoreLabs/uniffi-bindgen-java.

use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::HashMap;
use std::process::Command;
use uniffi_bindgen::{BindgenLoader, BindgenPaths};
use uniffi_bindgen_java_jni::generate_java_jni_bindings;
use uniffi_bindgen_java_jni::pipeline::config::JavaConfig;
use uniffi_testing::UniFFITestHelper;

/// Run an end-to-end test: build fixture → generate bindings → build Rust glue → compile Java → run.
fn run_test(fixture_package_name: &str, test_script: &str) -> Result<()> {
    let test_path = Utf8Path::new("tests").join(test_script);
    let test_helper = UniFFITestHelper::new(fixture_package_name)?;
    let out_dir = test_helper.create_out_dir(
        env!("CARGO_TARGET_TMPDIR"),
        &test_path,
    )?;
    let cdylib_path = test_helper.cdylib_path()?;

    // Find the fixture crate's root from cargo metadata (for --main-crate-path)
    let metadata = test_helper.cargo_metadata();
    let package = metadata
        .packages
        .iter()
        .find(|p| p.name == fixture_package_name)
        .with_context(|| format!("package '{fixture_package_name}' not found in cargo metadata"))?;
    let main_crate_path: Utf8PathBuf = package
        .manifest_path
        .parent()
        .context("no parent of manifest_path")?
        .into();

    // Build Java config — use a consistent package for all fixtures
    let config = JavaConfig {
        package_name: Some("uniffi.fixtures".to_string()),
        cdylib_name: Some(fixture_package_name.replace('-', "_")),
        custom_types: Default::default(),
    };

    // Build dependency overrides — if the main crate is in a git checkout
    // with a sibling uniffi directory, override the uniffi dep with a path.
    let mut dep_overrides: HashMap<String, String> = HashMap::new();
    {
        let git_uniffi = main_crate_path.join("..").join("..").join("uniffi").join("Cargo.toml");
        if git_uniffi.exists() {
            let uniffi_dir = std::fs::canonicalize(git_uniffi.parent().unwrap())
                .with_context(|| "failed to canonicalize uniffi dir")?;
            let mut dir_str = uniffi_dir.to_string_lossy().to_string();
            // Strip \\?\ prefix on Windows, use forward slashes for TOML
            if let Some(rest) = dir_str.strip_prefix(r"\\?\") {
                dir_str = rest.to_string();
            }
            dep_overrides.insert(
                "uniffi".to_string(),
                format!("{{ path = \"{}\" }}", dir_str.replace('\\', "/")),
            );
        }
    }

    // Generate bindings
    let java_out_dir = out_dir.join("java");
    let rust_out_dir = out_dir.join("rust-glue");

    let mut paths = BindgenPaths::default();
    paths.add_cargo_metadata_layer(false)?;
    let loader = BindgenLoader::new(paths);

    println!("Generating bindings for {fixture_package_name}...");
    generate_java_jni_bindings(
        &loader,
        &cdylib_path,
        &java_out_dir,
        &rust_out_dir,
        None,
        &config,
        Some(&main_crate_path),
        Some(fixture_package_name),
        &dep_overrides,
    )?;

    // Build Rust glue crate
    println!("Building Rust glue crate...");
    let status = Command::new("cargo")
        .arg("build")
        .current_dir(&rust_out_dir)
        .status()
        .context("Failed to spawn cargo build for Rust glue crate")?;
    if !status.success() {
        bail!("Rust glue crate build failed");
    }

    // Compile Java test
    println!("Compiling Java test...");
    let class_out_dir = out_dir.join("classes");
    fs_err::create_dir_all(&class_out_dir)
        .context("Failed to create Java class output directory")?;
    let status = Command::new("javac")
        .arg("-cp")
        .arg(java_out_dir.as_str())
        .arg("-d")
        .arg(class_out_dir.as_str())
        .arg(test_path.as_str())
        .status()
        .context("Failed to spawn javac")?;
    if !status.success() {
        bail!("Java compilation failed");
    }

    // Run Java test
    println!("Running Java test...");
    let native_lib_dir = rust_out_dir.join("target").join("debug");
    let compiled_name = test_path.file_stem().unwrap();
    let classpath_sep = if cfg!(windows) { ';' } else { ':' };
    let classpath = format!("{}{}{}", java_out_dir, classpath_sep, class_out_dir);

    let status = Command::new("java")
        .arg(format!("-Djava.library.path={}", native_lib_dir))
        .arg("-ea")
        .arg("-cp")
        .arg(&classpath)
        .arg(compiled_name)
        .status()
        .context("Failed to spawn java")?;
    if !status.success() {
        bail!("Java test failed");
    }

    Ok(())
}

// ── fixture_tests! macro ──────────────────────────────────────────────

macro_rules! fixture_tests {
    {$(
        $(#[$m:meta])*
        fn $test_name:ident($fixture:expr, $script:expr);
    )*} => {
        $(
            $(#[$m])*
            #[test]
            fn $test_name() -> Result<()> {
                run_test($fixture, $script)
            }
        )*
    };
}

fixture_tests! {
    /// Basic arithmetic: top-level functions with simple types.
    fn test_arithmetic("uniffi-example-arithmetic", "scripts/TestArithmetic.java");
}
