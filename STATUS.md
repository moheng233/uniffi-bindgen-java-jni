# 项目状态 — uniffi-bindgen-java-jni

**日期**: 2026-05-13
**编译**: ✅ 零错误零警告（clippy clean）
**Phase 1-5**: ✅ | **Phase 6**: ⏳ | **Phase 7**: ⚠️

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
| 5 | Rust 胶水模板 | ✅ | 5 个 Askama 模板激活，替换字符串拼接 |
| 6 | Callback Interface | ⏳ | 未开始（模板框架就位，逻辑 TODO） |
| 7 | CLI 完善 | ⚠️ | 参数解析可用，config 解析 TODO |

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
（当前 gen_rust 是字符串拼接，需重构为 Askama Template derive）

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
| `src/lib.rs` | ✅ | `generate_java_jni_bindings()` 完整 pipeline 调用链 |
| `src/main.rs` | ✅ | clap CLI（source/java-out-dir/rust-out-dir/config/crate-filter） |
| `src/pipeline/mod.rs` | ✅ | `general_pipeline()` 函数 |
| `src/pipeline/config.rs` | ✅ | `JavaConfig` / `CustomTypeConfig` |
| `src/pipeline/context.rs` | ✅ | `Context` 结构体 |
| `src/pipeline/nodes.rs` | ✅ | Java IR 全部节点，`RustFfiFunctionName` 含 `.name()` 方法 |
| `src/pipeline/modules.rs` | ✅ | 所有 `convert_*` 函数，general→Java 转换 |
| `src/pipeline/filters.rs` | ✅ | `ffi_type_java` / `java_type` Askama 过滤器 |
| `src/gen_java/mod.rs` | ✅ | `generate_java_code()` + `JavaCodeOracle`，渲染 wrapper.java |
| `src/gen_rust/mod.rs` | ✅ | 全部改用 Askama 模板渲染（含 `ffi_type_rust_name` 辅助函数） |
| `src/gen_rust/cargo_toml.rs` | ✅ | Askama：`CargoTomlTemplate` |
| `src/gen_rust/jni_types.rs` | ✅ | Askama：`JniTypesTemplate`（含 `raw_to_jni_bytebuffer` 辅助） |
| `src/gen_rust/jni_func.rs` | ✅ | `jni_func_name` / `jni_ctor_name` 工具函数 |
| `src/gen_rust/callback_gen.rs` | ✅ | Askama：`JniCallbackTemplate` |

### Java 模板（src/templates/java/）

| 文件 | 状态 | 内容 |
|------|------|------|
| `wrapper.java` | ✅ | 顶层模板，`{% include %}` 集成所有子模板 |
| `RustBuffer.java` | ✅ | ByteBuffer 包装器 + native handle 管理 |
| `RustBufferStream.java` | ✅ | 类型序列化流（read/write 各基本类型） |
| `HandleMap.java` | ✅ | 线程安全 handle→object 映射表 |
| `FfiConverter.java` | ✅ | FFI 转换器接口（lift/lower/read/write/allocationSize） |
| `Helpers.java` | ✅ | RustCallStatus + 错误处理辅助 |
| `Function.java` | ✅ | 顶层函数 include 模板 |
| `Object.java` | ✅ | Struct 对象 include 模板 |
| `Interface.java` | ✅ | Trait 对象 include 模板（接口 + Impl） |
| `Record.java` | ✅ | 数据记录 include 模板（含 equals/hashCode/toString） |
| `Enum.java` | ✅ | 枚举 include 模板（sealed class + 变体子类） |
| `CallbackInterface.java` | ✅ | 回调接口定义 include 模板 |
| `CallbackInterfaceImpl.java` | ✅ | 回调实现（HandleMap + 回调方法分发） |

### Rust 模板（src/templates/rust/）

| 文件 | 状态 | 内容 |
|------|------|------|
| `cargo_toml.rs` | ✅ | Cargo.toml 模板 |
| `lib.rs` | ✅ | lib.rs 模板（JNI_OnLoad + 模块声明） |
| `jni_bridge.rs` | ✅ | JNI→FFI 桥接函数模板 |
| `jni_types.rs` | ✅ | JNI 类型转换函数模板（含 raw_to_jni_bytebuffer） |
| `jni_callback.rs` | ✅ | 回调支持占位模板（Phase 6 待实现） |

### Stub 文件（待后续实现）

- `pipeline/callables, enums, records, interfaces, callback_interfaces, default, jni_signature, types`（8 个）
- `gen_java/primitives, compounds, object, callback_interface, enum_, record, custom, miscellany`（8 个）

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

1. **实现 Java 端 lift/lower/read/write 逻辑** — 模板框架就位但核心转换逻辑是 TODO
2. **实现 jni_bridge.rs 的函数体** — JNI→FFI 参数转换 + uniffi::ffi::rust_call 调用
3. **Callback Interface 完整实现** — Phase 6（VTable 生成，双向 JNI 调用）
4. **CLI config 解析** — `main.rs` 中有 TODO
5. **填充 16 个 pipeline/gen_java stub 文件** — 按需激活
6. **端到端测试** — fixture crate + Java 代码 + Gradle 编译

---

## 关键记忆（跨会话传递）

- **编译零错误零警告** — cargo check 和 clippy 均通过
- **不要碰 modules.rs 的 convert_* 函数** — 已验证，所有 general IR 字段/变体名正确
- **Module 是唯一用 `#[derive(Template)]` 的 Java 节点** — 其他 Java 类型通过 `{% include %}` 渲染
- **gen_rust 全部改用 Askama** — 5 个 Rust 模板全部激活，原字符串拼接已删除
- **RustFfiFunctionName 访问用 `.name()`** — Askama 模板中不能使用 `.0`
- **Askama 不能使用 `|` 闭包语法** — 用 `{% match %}` / `{% when %}` 代替 `.map_or("void", \|t\| t\|java_type)`
- **Java 模板 include 作用域** — 在 `{% match td %}{% when TypeDefinition::Object(obj) %}` 内 include 的模板可以直接访问 `obj`
- **askama.toml 需要 `escaper = "none"`** — 对 java 和 rs 语法都需要
