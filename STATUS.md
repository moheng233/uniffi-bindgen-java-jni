# 项目状态 — uniffi-bindgen-java-jni

**日期**: 2026-05-27
**编译**: ✅ 零错误零警告（clippy clean）| **测试**: ✅ 45 个测试全部通过
**Phase 1-7**: ✅ | **产物**: ✅ Java + Rust 胶水代码均零警告编译
**端到端**: ✅ `examples/simple/` Java 测试全部通过

---

## 指导原则（跨会话保持一致）

1. **不方案回退** — 坚持 Pipeline 架构（initial IR → general IR → Java IR），不退回 ComponentInterface
2. **解决问题优先** — 逐个修复，不重写模块
3. **不自主做决定** — 遇选择先问用户
4. **用 `cargo add` 加依赖** — 不要直接编辑 Cargo.toml

---

## 进度总览

| Phase | 内容 | 状态 | 说明 |
|-------|------|------|------|
| 1 | Cargo.toml + 模块结构 | ✅ | 全部就位 |
| 2 | Pipeline（config/nodes/modules/filters/context） | ✅ | general IR → Java IR 转换完成 |
| 3 | gen_java、gen_rust 入口层 | ✅ | 可编译、可调用 |
| 4 | Java 模板 | ✅ | 12 个模板全部填充，wrapper.java 使用 {% include %} 集成 |
| 5 | Rust 胶水模板 | ✅ | 5 个 Askama 模板激活，JNI 桥接实现 |
| 6 | Callback Interface + VTable | ✅ | VTable 结构体（repr(C)）生成，Rust→JNI→Java 回调链路完整 |
| 7 | CLI 完善 | ✅ | 参数解析、config 解析（`[bindings.java]` 段）、`--main-crate-path` 支持 |
| 8 | 使用案例 / 集成验证 | ✅ | `examples/simple/` 含 cdylib + UDL，Java 端到端测试通过 |

---

## 架构速查

```
用户 cdylib
    │
    ▼ BindgenLoader::load_metadata()
uniffi_meta metadata
    │
    ▼ load_pipeline_initial_root()
initial::Root
    │
    ▼ general_pipeline().execute()    (uniffi_bindgen::pipeline::general::pipeline("java"))
general::Root
    │
    ▼ convert_to_java_root()          (src/lib.rs → src/pipeline/modules.rs)
Java IR (src/pipeline/nodes.rs)
    │
    ├──▶ gen_java::generate_java_code()   → Askama 渲染 wrapper.java (含 12 个子模板) → {java_out_dir}
    └──▶ gen_rust::generate_rust_glue()   → Askama 渲染 Rust 模板 (5 个) → {rust_out_dir}
```
（当前 gen_rust 全部使用 Askama 模板渲染）

### 关键类型映射

| 方面 | general IR | Java IR (nodes.rs) |
|------|-----------|-------------------|
| 顶层 | `general::Root` | `nodes::Root`（手动转换） |
| 模块 | `general::Namespace` | `nodes::Module`（`convert_namespace()`） |
| 类型 | `general::Type`（含 `Interface` 变体） | `nodes::TypeNode`（含 `Object` 变体） |
| FFI | `general::FfiDefinition`（`RustFunction/FunctionType/Struct`） | `nodes::FfiDefinition`（`RustFunction/CallbackFunction`） |
| FFI 类型 | `general::FfiType`（无 Boolean/String/Bytes 变体） | `nodes::FfiType`（有 Boolean/String/Bytes） |

### 辅助函数（在 modules.rs 中）

- `convert_type_node(TypeNode) → TypeNode` — 从 general TypeNode 转 Java TypeNode
- `convert_type(Type) → TypeNode` — 从裸 general Type 转 Java TypeNode
- `convert_ffi_type_node(FfiTypeNode) → FfiType` — general FfiTypeNode → Java FfiType
- `convert_ffi_type(FfiType) → FfiType` — 裸 general FfiType → Java FfiType
- `to_upper_camel_case / to_lower_camel_case / to_snake_case` — 命名转换

### RustFfiFunctionName 访问

`RustFfiFunctionName(pub String)` 添加了 `.name()` 方法供 Askama 模板调用：
- 模板中：`{{ obj.ffi_func_free.name() }}`
- ❌ 不能：`{{ obj.ffi_func_free.0 }}`（Askama 不支持元组结构体字段访问）

---

## 各文件详细状态

### 核心文件（非 stub，有完整实现）

| 文件 | 状态 | 内容 |
|------|------|------|
| `Cargo.toml` | ✅ | 全部依赖配置完成 |
| `askama.toml` | ✅ | `escaper = "none"` 对 java + rs 语法 |
| `src/lib.rs` | ✅ | `generate_java_jni_bindings()` accepts `config: &JavaConfig` + `main_crate_path: Option<&Utf8Path>` |
| `src/main.rs` | ✅ | clap CLI（source/java-out-dir/rust-out-dir/config/crate-filter/main-crate-path），config 解析调用 `parse_java_config()` |
| `src/pipeline/mod.rs` | ✅ | `general_pipeline()` 函数 |
| `src/pipeline/config.rs` | ✅ | `JavaConfig` / `CustomTypeConfig` / `parse_java_config()` |
| `src/pipeline/context.rs` | ✅ | `Context` 结构体 |
| `src/pipeline/nodes.rs` | ✅ | Java IR 全部节点，`RustFfiFunctionName` 含 `.name()` 方法 |
| `src/pipeline/modules.rs` | ✅ | 所有 `convert_*` 函数，general→Java 转换 + 14 个单元测试 |
| `src/pipeline/filters.rs` | ✅ | `ffi_type_java` / `java_type` 等 6 个 Askama 过滤器 + 已提取 `_str` 纯函数 + 22 个单元测试 |
| `src/pipeline/body_gen.rs` | ✅ | Java 方法体预计算（lower→native→lift），模板用 `{{ func.body }}` 输出 + 9 个单元测试 |
| `src/pipeline/types.rs` | ✅ | `java_type_name()` / `java_boxed_type_name()` 类型名映射 |
| `src/pipeline/jni_signature.rs` | ✅ | JNI 方法签名字符串生成（`(Ljava/lang/String;I)J` 格式） |
| `src/gen_java/mod.rs` | ✅ | `generate_java_code()` + `JavaCodeOracle`，渲染 wrapper.java |
| `src/gen_rust/mod.rs` | ✅ | 全部改用 Askama 模板渲染（含 `ffi_type_rust_name` 辅助函数） |
| `src/gen_rust/cargo_toml.rs` | ✅ | Askama：`CargoTomlTemplate`（含 main_crate_path） |
| `src/gen_rust/jni_types.rs` | ✅ | Askama：`JniTypesTemplate`（`jni_bytebuffer_to_rustbuffer` 已改用 `uniffi_rustbuffer_alloc` FFI 分配） |
| `src/gen_rust/jni_func.rs` | ✅ | `jni_func_name` / `jni_ctor_name` 工具函数 |
| `src/gen_rust/callback_gen.rs` | ✅ | Askama：`JniCallbackTemplate`（提取 CallbackInterface 数据、生成 VTable + JNI 回调代码） |

### Java 模板（src/templates/java/）

| 文件 | 状态 | 内容 |
|------|------|------|
| `wrapper.java` | ✅ | 顶层模板，`{% include %}` 集成所有子模板 |
| `RustBuffer.java` | ✅ | ByteBuffer 包装器 + native handle 管理 + `readStringFromByteBuffer`/`readBytesFromByteBuffer` |
| `RustBufferStream.java` | ✅ | 类型序列化流（read/write 各基本类型） |
| `HandleMap.java` | ✅ | 线程安全 handle→object 映射表 |
| `FfiConverter.java` | ✅ | FFI 转换器接口（lift/lower/read/write/allocationSize） |
| `Helpers.java` | ✅ | RustCallStatus + 错误处理辅助 |
| `Function.java` | ✅ | 顶层函数 — 使用预计算 body（lower→native→lift） |
| `Object.java` | ✅ | Struct 对象 — 使用预计算 body |
| `Interface.java` | ✅ | Trait 对象 — Impl 方法使用预计算 body |
| `Record.java` | ✅ | 数据记录 — 包含完整 write()/read() 序列化逻辑 |
| `Enum.java` | ✅ | 枚举 — 包含 write()/read() 变体分发逻辑 |
| `CallbackInterface.java` | ✅ | 回调接口定义 + FfiConverter（lift/lower 通过 HandleMap） |
| `CallbackInterfaceImpl.java` | ✅ | 回调实现（HandleMap + 回调方法分发）

### Rust 模板（src/templates/rust/）

| 文件 | 状态 | 内容 |
|------|------|------|
| `cargo_toml.rs` | ✅ | Cargo.toml 模板（含 `once_cell` 条件依赖） |
| `lib.rs` | ✅ | lib.rs 模板（JNI_OnLoad → store_jvm + register_callbacks） |
| `jni_bridge.rs` | ✅ | JNI→FFI 桥接（`#![allow(unused_unsafe)]`，`needs_env` 条件化 env 参数，`return_conv_unsafe` 区分） |
| `jni_types.rs` | ✅ | JNI↔Rust 类型转换（RustBuffer 分配经 `uniffi_rustbuffer_alloc`） |
| `jni_callback.rs` | ✅ | VTable + Rust→JNI 回调函数（handle 管理、free/clone/method 回调、JNI 调用） |

---

## 关键依赖版本

```toml
uniffi_bindgen = "0.31.1"   # 从 crates.io，非 git
askama = "0.16.0"
clap = "4.6" (derive)
heck = "0.5"
indexmap = "2.14"
serde = "1.0" (derive)
```

## 重要参考文件（本机 cargo registry）

```
D:\packages\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\
├── uniffi_bindgen-0.31.1\src\
│   ├── pipeline\general\nodes.rs    ← general IR 实际类型定义
│   ├── pipeline\general\mod.rs      ← general::pipeline() 入口
│   ├── pipeline\initial\mod.rs      ← initial::Root 定义
│   ├── loader.rs                    ← BindgenLoader API
│   └── bindings\python\pipeline\    ← Python Pipeline 参考
└── uniffi_pipeline-0.31.1\src\
    └── pipeline.rs                  ← Pipeline/Node trait 定义
```

---

## 下一步工作建议（按优先级）

1. **Record 改用 Java 21 record** — 当前 `Record.java` 模板生成普通 class + getter，应迁移到 Java 21 `record` 关键字，符合 PLAN.md 的 Java 21 决策
2. **UniFFI fixture 测试集成** — 对 uniffi-rs 官方 fixture crate 运行生成器，验证覆盖所有类型（详见下方 §Fixture 测试集成计划）
3. **减少 body_gen.rs 预计算字符串** — 当前方法体由 Rust 端拼装字符串注入模板。未来 Askama 升级或切换到 Tera/minijinja 后可改为模板原生类型匹配
4. **RustCallStatus 错误处理润色** — `jni_bridge.rs` 中完善错误码传播

---

## Fixture 测试集成计划

### 背景

参考 [IronCoreLabs/uniffi-bindgen-java](https://github.com/IronCoreLabs/uniffi-bindgen-java) 的实现：
通过 Cargo 的 `[dev-dependencies]` 以 `git` 依赖方式直接引用 uniffi-rs 的 fixture crate，
无需克隆整个 uniffi-rs 仓库。Cargo 按需拉取源码，`cargo test` 一键运行全流程。

### 架构

```
Cargo.toml                              # [dev-dependencies] git 引用 fixture crate
│                                       # uniffi-fixture-coverall = { git = "...", tag = "v0.31.0" }
├── tests/
│   ├── fixture_tests.rs               # Rust 集成测试驱动（fixture_tests! 宏）
│   └── scripts/                       # Java 测试文件
│       ├── TestFixtureCoverall.java
│       ├── TestCallbacks.java
│       └── ...
└── fixtures/                          # 仅放本项目自定义 fixture（如有）
```

每条测试流程（在 `tests/fixture_tests.rs` 中）：
1. `uniffi_testing::UniFFITestHelper::new(fixture_name)` 构建 fixture cdylib
2. 调用 `generate_java_jni_bindings()` 生成 Java + Rust 胶水
3. `javac -Werror` 编译生成的 Java + 测试文件
4. `java -Djava.library.path=...` 运行测试

### 优先级 fixture（引用 uniffi-rs）

| Fixture crate | 覆盖类型 | 优先级 | Java 测试文件 |
|---------|---------|--------|--------------|
| `uniffi-fixture-coverall` | 全部：Primitives, Optional, Record, Enum, Object, Trait, Callback, Error, Dict, Sequence, Map | P0 | `TestFixtureCoverall.java` |
| `uniffi-example-callbacks` | Callback Interface | P0 | `TestCallbacks.java` |
| `uniffi-example-arithmetic` | 顶层函数 | P1 | `TestArithmetic.java` |
| `uniffi-example-geometry` | Object/Interface | P1 | `TestGeometry.java` |
| `uniffi-example-rondpoint` | 复杂类型往返 | P1 | `TestRondpoint.java` |
| `uniffi-example-sprites` | Object + 方法 | P1 | `TestSprites.java` |
| `uniffi-fixture-time` | Timestamp/Duration | P1 | `TestChronological.java` |
| `uniffi-example-custom-types` | 自定义类型 | P2 | `TestCustomTypes.java` |
| `uniffi-fixture-ext-types` | 外部类型引用 | P2 | `TestImportedTypes.java` |
| `uniffi-fixture-trait-methods` | Trait 方法 | P2 | `TestTraitMethods.java` |
| `uniffi-fixture-proc-macro` | proc-macro 方式 | P2 | `TestProcMacro.java` |
| `uniffi-fixture-rename` | 重命名 | P2 | `TestRename.java` |

### 实施步骤

| Step | 操作 | 说明 |
|------|------|------|
| 1 | 在 `Cargo.toml` 添加 `[dev-dependencies]` | `git` 引用 uniffi-rs fixture crate + `uniffi_testing`（tag = v0.31.1） |
| 2 | 创建 `tests/fixture_tests.rs` | 参照 IronCoreLabs 的 `fixture_tests!` 宏模式 |
| 3 | 创建 `tests/scripts/` 目录 + Java 测试文件 | 先从 `TestFixtureCoverall.java` 开始（参考 `test_coverall.kts`） |
| 4 | 实现 `run_test()` 辅助函数 | build cdylib → generate → javac → java |
| 5 | 逐步添加更多 fixture | 按 P0 → P1 → P2 优先级扩展

### 与现有 examples/simple/ 的关系

| 方面 | `examples/simple/` | fixture 集成 |
|------|-------------------|-------------|
| 目的 | 演示项目用法 | 回归测试套件 |
| 类型覆盖 | 基本类型 ~60% | 完整覆盖 ~95% |
| 运行方式 | 手动命令行 | `cargo test` 自动化 |
| Error 类型 | 不支持 | 多种 Error 类型 |
| External types | 不支持 | 支持 |
| Trait methods | 不支持 | 支持 |

---

## 关键记忆（跨会话传递）

### 架构约定
- **不要碰 modules.rs 的 convert_* 函数** — 已验证，所有 general IR 字段/变体名正确
- **Pipeline 不可回退** — 坚持 initial IR → general IR → Java IR，不退回 ComponentInterface
- **Module 是唯一用 `#[derive(Template)]` 的 Java 节点** — 其他 Java 类型通过 `{% include %}` 渲染
- **gen_rust 全部使用 Askama** — 5 个 Rust 模板激活，无字符串拼接

### Askama 限制与对策
- **`RustFfiFunctionName` 用 `.name()`** — 模板中不能 `.0`（Askama 不支持元组字段访问）
- **不能用 `|` 闭包语法** — 用 `{% match %}` / `{% when %}` 代替 `.map_or("void", |t| ...)`
- **避免嵌套 match 含 filter** — `{% match ret|filter %}` 可能触发类型错误
- **方法体用 Rust 预计算** — `body_gen.rs` 生成 body，模板用 `{{ func.body }}`，避免 Askama 类型匹配问题
- **⚠️ TODO: 减少预计算字符串** — 理想方案是模板自行处理类型匹配，受限于 tuple variant 和嵌套 match+filter 支持。未来可选：(1) 升级 Askama; (2) 自定义 `#[derive(Template)]` render; (3) 切换到 Tera/minijinja

### Java 模板约定
- **include 作用域** — `{% match td %}{% when TypeDefinition::Object(obj) %}` 内 include 的模板可直接访问 `obj`
- **内嵌类用 `static`** — RustBuffer、RustBufferStream、HandleMap、Helpers 均为 `public static final class`
- **`askama.toml` 需要 `escaper = "none"`** — 对 java 和 rs 语法都要

### JNI 桥接
- **FFI 调用 unsafe** — FFI 函数是 unsafe fn，模板中包装 `unsafe { }`
- **零警告策略** — `jni_bridge.rs` 模板：`#![allow(unused_unsafe)]`；`needs_env` 条件化 `mut env` / `_env`；`return_conv_unsafe` 仅 buffer 返回包装 unsafe
- **JNI 方法名下划线转义** — `_` → `_1` 以合 JNI 命名规范
- **无符号类型参数收窄** — UInt8→`(byte)`、UInt16→`(short)`、UInt32→`(int)`
- **String/Bytes lowering 返回 RustBuffer** — native 调用处 `.asByteBuffer()` 转换
- **对象 free/clone 不过滤** — `_fn_free_` 和 `_fn_clone_` 正常生成桥接代码

### RustBuffer 转换
- **分配走 FFI** — `jni_bytebuffer_to_rustbuffer` 用 `uniffi::ffi::uniffi_rustbuffer_alloc` + `copy_nonoverlapping`，不直接用 `RustBuffer::from_vec()`
- **`RustBuffer.data` 是 `pub(crate)`** — 写入用 `data_pointer() as *mut u8`

### VTable 回调
- **VTable 结构体** — 胶水库自行定义 `#[repr(C)]` 结构体（含 `Option<unsafe extern "C" fn(...)>` 字段），不依赖主 crate 导出类型；free/clone 字段用具体函数签名，方法字段用 `*const c_void`
- **回调注册** — `JNI_OnLoad` → `store_jvm(&vm)` → `register_callbacks()` → 调用 `extern "C" { fn init_callback_vtable_*(vtable: *const c_void) }`
- **JVM 存储** — 用 `static mut JVM_PTR: *mut JavaVM` 存储原始指针（避免 `Clone` trait 问题）；`get_jvm()` 通过 `JavaVM::from_raw()` 恢复
- **回调流程** — Rust 调用 VTable 函数指针 → 胶水库 `extern "C"` 回调函数 → `attach_current_thread()` → 通过 handle 查找 GlobalRef → JNI `call_static_method` → Java `callback*` 方法 → Java HandleMap 查找实现
- **JNI 对象生命周期** — String/Buffer 类型通过 let 绑定保持 JNI 对象存活至 JValue 数组引用结束后

### 类型转换（modules.rs）
- **`TypeDefinition::Simple/Optional/Sequence/Map/Custom/External`** → `convert_type_definition` 返回 `Ok(None)`（不需要独立 Java 类）
- **`FfiDefinition::Struct`** → `convert_ffi_definition` 返回 `Ok(None)`（VTable struct 由 callback 处理）
- **`Argument` 含 `lower_code` 和 `native_expr`** — 在 `convert_argument()` 中通过 `body_gen` 预计算
- **Record/Enum 有 `write_body`/`read_body`** — 序列化逻辑在 Rust 端预计算

### 产物格式
- **末尾换行** — `normalize_content()` 修剪尾部空白，确保 TOML/Rust 文件末尾格式正确
- **Cargo.toml 路径** — 主 crate 依赖路径用正斜杠（TOML 兼容），Rust `use` 用下划线形式 crate 名

### 开发与测试
- **使用案例**: `examples/simple/` — cdylib + UDL，运行 pipeline 生成胶水代码
- **测试入口**: `TestSimple.java` — 顶层函数、Calculator 对象、Record、Enum 完整验证
- **fixture 测试**（计划中）: `tests/fixture_tests.rs` — 集成测试驱动（`fixture_tests!` 宏）；`tests/scripts/Test*.java` — Java 测试文件；fixture crate 通过 `[dev-dependencies]` git 引用
- **运行命令**:
  ```bash
  # 生成胶水代码
  cargo run -- --source examples/simple/src/simple.udl --java-out-dir examples/simple/generated/java --rust-out-dir examples/simple/generated/rust-glue --config examples/simple/uniffi.toml --main-crate-path examples/simple
  # 构建胶水库
  cd examples/simple/generated/rust-glue && cargo build
  # 运行 Java 测试
  cd examples/simple && javac -cp "generated/java" TestSimple.java && java "-Djava.library.path=generated/rust-glue/target/debug" -cp "generated/java;." TestSimple
  ```
