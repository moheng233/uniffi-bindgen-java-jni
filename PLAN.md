# Plan: uniffi-bindgen-java-jni — Java JNI 绑定生成器

**TL;DR**: 为 uniffi-rs 实现独立的 Java JNI 外部绑定生成器 crate。**Pipeline 架构**（initial→general→Java IR），**Askama 双模板**渲染（Java + Rust），**双产物输出**（Java native 代码 + Rust JNI 胶水库），Java 21，零外部依赖，完整支持 Callback Interface。

---

## 总体架构

```
uniffi-bindgen CLI
    │
    ├── 读取 cdylib 元数据 (BindgenLoader)
    ├── Pipeline: initial::Root → general::Root → Java::Root
    │
    ├── 输出 1: Java 代码 (--java-out-dir)
    │   ├── 单一 wrapper.java（{% include %} 集成所有类型）
    │   ├── native 方法声明 (System.loadLibrary)
    │   ├── ByteBuffer 类型转换
    │   └── 零外部依赖
    │
    └── 输出 2: Rust JNI 胶水库 (--rust-out-dir)
        ├── Cargo.toml (依赖主 crate + jni crate)
        ├── src/lib.rs (JNI_OnLoad + native 方法实现)
        ├── src/jni_bridge.rs (JNI → UniFFI FFI 转发)
        ├── src/jni_types.rs (Java 类型 ↔ Rust 类型转换)
        └── src/jni_callback.rs (JNI 回调 Java 实现)
```

---

## Phase 1: 项目基础设施

**Step 1** — 配置 `Cargo.toml` 依赖

- `uniffi_bindgen`（pipeline, BindgenLoader, BindgenPaths, GlobalConfig）
- `uniffi_pipeline` + `uniffi_internal_macros`（Node, MapNode, Pipeline 宏）
- `askama`（渲染 Java + Rust 模板）
- `heck`, `anyhow`, `camino`, `serde`, `toml`, `fs-err`

**Step 2** — 创建模块结构

```
src/
├── main.rs                       # CLI 入口
├── lib.rs                        # 公共 API
├── pipeline/                     # Java IR 管线
│   ├── mod.rs                    # pipeline() 函数
│   ├── context.rs                # Java 特定 Context
│   ├── nodes.rs                  # Java IR 节点
│   ├── modules.rs               # namespace → Module 映射（核心转换逻辑）
│   ├── config.rs                 # JavaConfig
│   ├── body_gen.rs              # Java 方法体预计算（lower→native→lift）
│   ├── filters.rs               # Askama 过滤器（ffi_type_java, java_type 等）
│   ├── types.rs                  # Type 映射
│   └── jni_signature.rs         # JNI 方法签名生成
├── gen_java/                     # Java 端代码生成
│   └── mod.rs                    # generate_java_code() + JavaCodeOracle
├── gen_rust/                     # Rust 胶水端代码生成
│   ├── mod.rs                    # Rust 生成入口 + 类型映射辅助
│   ├── jni_func.rs              # JNI 函数命名
│   ├── jni_types.rs             # JNI 类型转换模板
│   ├── callback_gen.rs          # Callback Interface 模板
│   └── cargo_toml.rs            # Cargo.toml 模板
├── templates/
│   ├── java/                     # Java 端模板 (13 个)
│   │   ├── wrapper.java          # 顶层模板，{% include %} 集成所有子模板
│   │   ├── Function.java
│   │   ├── Object.java
│   │   ├── Interface.java
│   │   ├── Record.java
│   │   ├── Enum.java
│   │   ├── CallbackInterface.java
│   │   ├── CallbackInterfaceImpl.java
│   │   ├── FfiConverter.java
│   │   ├── RustBuffer.java
│   │   ├── RustBufferStream.java
│   │   ├── HandleMap.java
│   │   └── Helpers.java
│   └── rust/                     # Rust 胶水库模板 (5 个)
│       ├── cargo_toml.rs
│       ├── lib.rs
│       ├── jni_bridge.rs
│       ├── jni_types.rs
│       └── jni_callback.rs
```

## Phase 2: Pipeline — 通用 IR → Java IR

*与 Phase 3 可并行*

**Step 3** — `pipeline/config.rs` + `pipeline/context.rs`

- `JavaConfig`: package_name, cdylib_name, custom_types
- `Context`: 持有 JavaConfig, 当前 namespace/crate/type

**Step 4** — `pipeline/nodes.rs`

- `use_prev_node!` 从 general IR 继承
- 为 Java 特定节点标注 `#[derive(Template)]`
- Module 新增 package_name, imports, native_methods

**Step 5** — `pipeline/mod.rs`

```rust
pub fn general_pipeline() -> Pipeline<initial::Root, general::Root> {
    uniffi_bindgen::pipeline::general::pipeline("java")
}
```

> 注：Java IR 转换通过手动函数 `convert_namespace()` 等完成，而非 Pipeline pass。

**Step 6** — `pipeline/modules.rs`

- general::Namespace → Java Module（核心转换逻辑，含 14 个单元测试）
- 计算 Java imports + JNI native 方法签名列表 + Rust 胶水函数列表

**Step 6a** — `pipeline/body_gen.rs`（实际实现中新增）

- Java 方法体预计算（lower→native→lift），解决 Askama tuple variant 匹配限制

**Step 6b** — `pipeline/filters.rs`（实际实现中新增）

- 6 个 Askama 过滤器：`ffi_type_java`, `java_type`, `lower_code`, `native_arg`, `return_category`, `lift_code`

## Phase 3: 代码生成入口层

**Step 7** — `gen_java/mod.rs`

- `generate_java_code()` — 调用 Askama 渲染 `wrapper.java`
- `JavaCodeOracle`: class_name→UpperCamelCase, fn_name/var_name→lowerCamelCase

**Step 8** — `gen_rust/mod.rs`

- `generate_rust_glue()` — 协调 5 个 Rust 模板渲染
- JNI 类型签名生成（`ffi_type_to_jni_name`, `ffi_type_to_ffi_rust_name`）
- Cargo.toml 路径计算（`path_relative_to`）

## Phase 4: Java 端模板

*依赖 Step 7, 与 Phase 5 可并行*

**Step 9** — `RustBuffer.java` — ByteBuffer 包装器

- `ByteBuffer.allocateDirect()`
- allocFromString/allocFromBytes/consumeIntoString/consumeIntoBytes

**Step 10** — `RustBufferStream.java` — 类型读写流

- readInt8..64, writeInt8..64 等基本方法
- 为每种 UniFFI 类型生成 read{Type}/write{Type}

**Step 11** — `HandleMap.java`

- ConcurrentHashMap<Long, T> + AtomicLong
- insert/get/remove/cloneHandle

**Step 12** — `FfiConverter.java`

- interface FfiConverter<JavaType, FfiType>
- lift/lower/read/write/allocationSize

**Step 13** — `Helpers.java`

- RustCallStatus POJO (int8 code + ByteBuffer errorBuf)
- rustCall/rustCallWithError 辅助

**Step 14** — `wrapper.java`

- package 声明 + 零外部依赖 imports
- include 子模板 + 静态块 `System.loadLibrary()`
- 集中声明所有 `private static native` 方法

**Step 15** — `Function.java` — 顶层函数

- public static 方法, 参数 lower→native→返回值 lift

**Step 16** — `Object.java` — Rust 对象

- 持有 long handle, AutoCloseable
- 构造器/方法/析构

**Step 17** — `Record.java` — Java 21 record

- RustBuffer 序列化/反序列化
- FfiConverterRecord

**Step 18** — `Enum.java` — 枚举

- 带数据 variant→嵌套 static 类
- RustBuffer 序列化

**Step 19** — `CallbackInterface.java` — 回调接口定义

- Java interface 声明
- FfiConverterCallbackInterface<T> 基类
- register(lib) 静态方法

**Step 20** — `CallbackInterfaceImpl.java` — VTable + 注册

- 每个方法→内部 Callback handler 类
- 参数 lower→调 Java 实现→返回值 lift
- try/catch→RustCallStatus 错误处理

## Phase 5: Rust JNI 胶水库模板

*依赖 Step 8, 与 Phase 4 可并行*

**Step 21** — `templates/rust/cargo_toml.rs`

```toml
[package]
name = "uniffi-jni-glue-{crate_name}"
[lib] crate-type = ["cdylib"]
[dependencies]
jni = "0.21"
uniffi = "..."
{crate_name} = { path = "{main_crate_path}" }
```

**Step 22** — `templates/rust/jni_types.rs`

- jni_to_rust_string / rust_to_jni_string
- jni_bytebuffer ↔ rustbuffer 转换
- 基本类型零开销转换

**Step 23** — `templates/rust/jni_bridge.rs`

- 每个 UniFFI FFI 函数→`#[no_mangle] pub extern "system" fn Java_xxx`
- JNI params→Rust 类型, 调 uniffi FFI, 返回→JNI 类型
- 使用 `uniffi::ffi::rust_call` 处理错误

**Step 24** — `templates/rust/jni_callback.rs`

- Rust 端持有 `JNIEnv` + `GlobalRef` 到 Java 回调对象
- 每个 callback 方法→`extern "C" fn callback_xxx`
- 类型转换→调 Java 方法→返回转换→错误处理

**Step 25** — `templates/rust/lib.rs`

- JNI_OnLoad: 存储 JavaVM 指针
- 初始化: 调用 UniFFI init callback 注册 VTable
- 回调引用管理: HashMap<u64, GlobalRef>

## Phase 6: Callback Interface 完整实现

*依赖 Step 19-20 (Java) + Step 24-25 (Rust)*

**Step 26** — VTable 生成 (Java+Rust 配合)

- Rust: repr(C) VTable struct
- Java: 创建 VTable 实例传给 Rust
- 生命周期: Java对象↔handle映射, drop→通知Rust

**Step 27** — Callback 错误处理

- Java 抛异常→JNI Rust 端 catch→RustCallStatus 错误码
- 区分预期错误 vs 意外错误
- 遵循 UniFFI callback 规范 (UNIFFI_CALLBACK_SUCCESS/ERROR/UNEXPECTED_ERROR)

## Phase 7: CLI 和公共 API

*依赖所有前序 Phase*

**Step 28** — `main.rs` CLI

- clap 参数: --source, --java-out-dir, --rust-out-dir, --config, --crate, --main-crate-path
- 流程: BindgenLoader→pipeline→渲染 Java 模板→渲染 Rust 模板

**Step 29** — `lib.rs` 公共 API

- `generate_java_jni_bindings(loader, java_out_dir, rust_out_dir, main_crate_path)`
- `generate_java_code(root, out_dir)` / `generate_rust_glue(root, out_dir, main_crate_path)`

**Step 30** — 配置文件解析

- uniffi.toml `[bindings.java]`: package_name, cdylib_name, custom_types

## Phase 8: 验证

**Step 31** — 基本类型往返测试 (fixture crates)
**Step 32** — Callback Interface 测试 (Java→Rust→Java)
**Step 33** — Object 生命周期测试
**Step 34** — 端到端集成测试 (Gradle + Cargo)

---

## Decisions

| 决策 | 选择 | 理由 |
|------|------|------|
| FFI 方式 | JNI | 零外部依赖, 性能优先, 标准 Java |
| 产物 | Java 代码 + Rust 胶水库 | 双输出目录, 用户自行编译 |
| 集成方式 | 独立外部 crate | 不合并进 uniffi-rs |
| 架构 | Pipeline (initial→general→Java IR) | 复用通用 IR |
| Java 版本 | Java 21 | record 类型等 |
| RustBuffer | java.nio.ByteBuffer (direct) | 零拷贝 JNI 访问 |
| Cargo 依赖 | 胶水库依赖主 crate + jni | 用户编译为 cdylib |
| 文件结构 | 单一 wrapper.java + {% include %} 子模板 | 简化文件管理，所有类型在一个 Java 文件中 |
| Async | 预留 TODO | 第一期同步 |
| Callback | 完整支持 | 双向 JNI 调用 |
| 配置 | package_name + cdylib_name + custom_types | |

---

## Relevant files

- `d:\Project\uniffi-bindgen-java-jna\Cargo.toml` — 需配置依赖
- `d:\Project\uniffi-bindgen-java-jna\src\main.rs` — CLI 入口
- `moheng233/uniffi-rs` `uniffi_bindgen/src/bindings/python/pipeline/` — Pipeline 参考
- `moheng233/uniffi-rs` `uniffi_bindgen/src/bindings/kotlin/gen_kotlin/` — CodeType 模式参考
- `moheng233/uniffi-rs` `uniffi_bindgen/src/bindings/kotlin/templates/` — FFI 概念参考 (实现改为 JNI)
- `jni` crate (https://docs.rs/jni) — JNI Rust 端 API

---

## Further Considerations

1. **主 crate 路径**: 需要 `--main-crate-path` 指定主 crate 根目录作为胶水库的依赖路径
2. **库名约定**: `System.loadLibrary("{name}")` → Linux `lib{name}.so`, macOS `lib{name}.dylib`, Windows `{name}.dll`
3. **外部类型**: 第一期是否需要跨 crate 外部类型引用?
4. **格式化**: 生成后是否调用 `google-java-format`? (可提供 `--format` 选项)
