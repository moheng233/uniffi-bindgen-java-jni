# uniffi-bindgen-java-jna

为 [UniFFI](https://github.com/mozilla/uniffi-rs) 实现独立的 Java JNI 外部绑定生成器。

从 Rust UDL/CDylib 中读取 UniFFI 元数据，经 Pipeline 架构（initial IR → general IR → Java IR）转换，通过 Askama 模板渲染，**双产物输出**——
生成 Java native 代码及 Rust JNI 胶水库，使 Java 可通过 JNI 直接调用 Rust 导出的 UniFFI 接口。

## 架构

```
用户 cdylib / UDL
    │  BindgenLoader::load_metadata()
uniffi_meta 元数据
    │  general_pipeline()
general IR
    │  convert_to_java_root()
Java IR  ──┬──▶ gen_java  (Askama → Java 源文件)
           └──▶ gen_rust  (Askama → Rust JNI 胶水库)
```

## 使用

### 1) 编写 UDL 及 Rust 实现

参见 `examples/simple/` —— 含 `src/simple.udl`、`src/lib.rs`、`Cargo.toml`、`build.rs`。

### 2) 运行生成器

```powershell
cargo run -- `
  --source examples/simple/src/simple.udl `
  --config examples/simple/uniffi.toml `
  --java-out-dir examples/simple/generated/java `
  --rust-out-dir examples/simple/generated/rust-glue `
  --main-crate-path examples/simple
```

- `--source`：UDL 文件路径
- `--config`：uniFFI 配置文件，含 `[bindings.java]` 段（package_name、cdylib_name）
- `--java-out-dir`：Java 代码输出目录
- `--rust-out-dir`：Rust JNI 胶水库输出目录
- `--main-crate-path`：主 crate 根目录（胶水库 Cargo.toml 依赖之）

### 3) 构建胶水库

```powershell
cd examples/simple/generated/rust-glue
cargo build
```

产物为 `target/debug/uniffi_example_simple.dll`（Linux: `.so`，macOS: `.dylib`）。

### 4) 编译并运行 Java 测试

```powershell
cd examples/simple
javac -cp generated/java -d . TestSimple.java
java "-Djava.library.path=generated/rust-glue/target/debug" `
     -cp "generated/java;." TestSimple
```

输出示例：

```
=== UniFFI JNI 绑定测试 ===

--- 1. 顶层函数 ---
add(10, 20) = 30
multiply(6, 7) = 42
greet("世界") = Hello, 世界!

--- 2. Calculator 对象 ---
calc.add(50) = 150
calc.getValue() = 120
calc.processData(MyData(5, "测试数据")) = MyData(125, "processed: 测试数据")

    ...
   全部测试通过！🎉
```

## 配置 (uniffi.toml)

```toml
[bindings.java]
package_name = "com.example.uniffi"
cdylib_name = "uniffi_example_simple"
```

| 项 | 说明 | 默认值 |
|---|---|---|
| `package_name` | Java package 名 | `uniffi.{namespace}` |
| `cdylib_name` | `System.loadLibrary()` 加载的库名 | namespace 名 |
| `custom_types` | 自定义类型映射（可选） | 无 |

## 项目状态

- **编译**：零错误
- **测试**：45 个 Rust 单元测试 + Java 端到端测试，全部通过
- **支持类型**：原语、String、Bytes、Record、Enum、Object（含构造器/方法/析构）、Callback Interface（部分）
- **平台**：Windows / Linux / macOS
