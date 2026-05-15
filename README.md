# uniffi-bindgen-java-jni

[![License](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](rust-toolchain.toml)
[![Java](https://img.shields.io/badge/java-21%2B-red.svg)](#)
[![Status](https://img.shields.io/badge/status-active-brightgreen.svg)](STATUS.md)
[![Tests](https://img.shields.io/badge/tests-45%20passed-success.svg)](#)
[![Clippy](https://img.shields.io/badge/clippy-clean-success.svg)](#)

A standalone Java JNI external binding generator for [UniFFI](https://github.com/mozilla/uniffi-rs).

Reads UniFFI metadata from Rust UDL/cdylib, transforms it through a Pipeline architecture
(initial IR → general IR → Java IR), renders via Askama templates, and produces **dual output** —
Java native source files and a Rust JNI glue crate — enabling Java to call Rust-exported
UniFFI interfaces directly through JNI.

> **Repository**: [moheng233/uniffi-bindgen-java-jni](https://github.com/moheng233/uniffi-bindgen-java-jni)

For the Chinese version, see [README.cn.md](README.cn.md).

## Architecture

```
User cdylib / UDL
    │  BindgenLoader::load_metadata()
uniffi_meta metadata
    │  general_pipeline()
general IR
    │  convert_to_java_root()
Java IR  ──┬──▶ gen_java  (Askama → Java source files)
           └──▶ gen_rust  (Askama → Rust JNI glue crate)
```

## Supported Types

| Category | Types |
|----------|-------|
| Primitives | `i8`–`i64`, `u8`–`u64`, `f32`, `f64`, `bool` |
| Strings & Bytes | `string`, `bytes` |
| Records | `dictionary` / `record` with full `write()`/`read()` serialization |
| Enums | Flat enums and enums with associated data |
| Objects | Constructors, methods, destructors (free), clone |
| Callback Interfaces | Trait implemented by Java, called from Rust — full VTable + JNI round-trip |

## Usage

### 1) Write UDL and Rust implementation

See `examples/simple/` — contains `src/simple.udl`, `src/lib.rs`, `Cargo.toml`, `build.rs`.

### 2) Run the generator

```powershell
cargo run -- `
  --source examples/simple/src/simple.udl `
  --config examples/simple/uniffi.toml `
  --java-out-dir examples/simple/generated/java `
  --rust-out-dir examples/simple/generated/rust-glue `
  --main-crate-path examples/simple
```

| Flag | Description |
|------|-------------|
| `--source` | Path to the UDL file |
| `--config` | UniFFI config file with `[bindings.java]` section (`package_name`, `cdylib_name`) |
| `--java-out-dir` | Output directory for Java source files |
| `--rust-out-dir` | Output directory for the Rust JNI glue crate |
| `--main-crate-path` | Root directory of the main crate (used as dependency in glue Cargo.toml) |
| `--crate` | (Optional) Limit generation to a single crate |

### 3) Build the glue crate

```powershell
cd examples/simple/generated/rust-glue
cargo build
```

Produces `target/debug/uniffi_example_simple.dll` (`.so` on Linux, `.dylib` on macOS).

### 4) Compile and run Java tests

```powershell
cd examples/simple
javac -cp generated/java -d . TestSimple.java
java "-Djava.library.path=generated/rust-glue/target/debug" `
     -cp "generated/java;." TestSimple
```

Sample output:

```
=== UniFFI JNI Binding Test ===

--- 1. Top-level Functions ---
add(10, 20) = 30
multiply(6, 7) = 42
greet("World") = Hello, World!

--- 2. Calculator Object ---
calc.add(50) = 150
calc.getValue() = 120
calc.processData(MyData(5, "test")) = MyData(125, "processed: test")

--- 6. Callback Interface (CalculatorListener) ---
FfiConverter.lower(listener) → handle = 1
  [callback] onCalculation("add", 42)
  [callback] onCalculation("multiply", 99)

========================================
   All tests passed! 🎉
========================================
```

## Configuration (uniffi.toml)

```toml
[bindings.java]
package_name = "com.example.uniffi"
cdylib_name = "uniffi_example_simple"
```

| Key | Description | Default |
|-----|-------------|---------|
| `package_name` | Java package name | `uniffi.{namespace}` |
| `cdylib_name` | Library name for `System.loadLibrary()` | Namespace name |
| `custom_types` | Custom type mappings (optional) | None |

## Project Status

- **Build**: 0 errors, 0 warnings (clippy clean)
- **Tests**: 45 Rust unit tests + Java end-to-end tests, all passing
- **Platforms**: Windows / Linux / macOS
- **Java**: 21+ (requires `java.nio.ByteBuffer` direct buffers)
- **Rust**: nightly toolchain (required by UniFFI 0.31)

## License

This project is a standalone external binding generator for UniFFI.
See [UniFFI](https://github.com/mozilla/uniffi-rs) for upstream licensing.
