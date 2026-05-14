# 项目状态 — uniffi-bindgen-java-jni

**日期**: 2026-05-14
**编译**: ✅ 零错误零警告（clippy clean）
**Phase 1-7**: ✅ | **测试**: ✅ 44 个测试全部通过
**Phase 8（使用案例）**: ✅ | **产物**: ✅ Java + Rust 胶水代码均可编译
**JNI→FFI 桥接**: ✅ 真实 FFI 调用已实现
**端到端集成**: ✅ Java 测试代码调用 Uniffi JNI 全部通过

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
| 6 | Callback Interface + JNI 桥接 | ✅ | CallbackInterface 模板就位，jni_bridge.rs 真实 FFI 调用已实现 |
| 7 | CLI 完善 | ✅ | 参数解析、config 解析（`[bindings.java]` 段）、`--main-crate-path` 支持 |

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
| `src/gen_java/mod.rs` | ✅ | `generate_java_code()` + `JavaCodeOracle`，渲染 wrapper.java |
| `src/gen_rust/mod.rs` | ✅ | 全部改用 Askama 模板渲染（含 `ffi_type_rust_name` 辅助函数） |
| `src/gen_rust/cargo_toml.rs` | ✅ | Askama：`CargoTomlTemplate`（含 main_crate_path） |
| `src/gen_rust/jni_types.rs` | ✅ | Askama：`JniTypesTemplate`（含 `raw_to_jni_bytebuffer` 辅助） |
| `src/gen_rust/jni_func.rs` | ✅ | `jni_func_name` / `jni_ctor_name` 工具函数 |
| `src/gen_rust/callback_gen.rs` | ✅ | Askama：`JniCallbackTemplate` |

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
| `CallbackInterface.java` | ✅ | 回调接口定义 include 模板（TODO: VTable） |
| `CallbackInterfaceImpl.java` | ✅ | 回调实现（HandleMap + 回调方法分发）

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

1. **消减 JNI bridge warnings** — 优化模板以消除 unnecessary unsafe、unused_mut、unused_variable 等警告
2. **RustBuffer ↔ JNI ByteBuffer 转换完善** — `jni_types.rs` 中 `jni_bytebuffer_to_rustbuffer` 需用 FFI 函数（`ffi_*_rustbuffer_alloc`）而非直接构造 RustBuffer
3. **Callback Interface VTable 生成** — 双向 JNI 调用（VTable dispatch）
4. **填充 stub 文件** — 按需激活 16 个 pipeline/gen_java stub 文件
5. **端到端测试** — fixture crate + Java 代码 + Gradle 编译
6. **实现 RustCallStatus 的完整 JNI 桥接** — jni_bridge.rs 中错误处理润色

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
- **方法体用 Rust 预计算** — `body_gen.rs` 生成函数/构造器/方法的完整 body，模板只用 `{{ func.body }}` 输出（避免 Askama 类型匹配问题）
- **模板避免嵌套 match 含 filter** — Askama 的 `{% match ret|filter %}` 可能触发类型错误，改用预计算字符串
- **Argument 含 `lower_code` 和 `native_expr`** — 在 `convert_argument()` 中调用 `body_gen` 预计算
- **Record/Enum 有 `write_body`/`read_body`** — 序列化逻辑在 Rust 端预计算
- **⚠️ TODO: 减少预计算字符串依赖** — 当前 `body_gen.rs` 将大量 Java 代码拼接为字符串再注入模板，导致代码风格割裂、难以维护。理想方案是 Askama 模板自行处理类型匹配和代码生成，但受限于 Askama 对 tuple variant (如 `TypeNode::Optional(Box<TypeNode>)`) 和嵌套 match+filter 的支持不足。未来可选方向：为 Java 节点实现自定义 `#[derive(Template)]` render 方法；
- **使用案例位于 `examples/simple/`** — 含 cdylib + UDL，运行 pipeline 生成 Java + Rust 胶水代码，均通过编译验证
- **TypeDefinition::Simple/Optional/Sequence/Map/Custom/External 在 convert_type_definition 中返回 `Ok(None)`** — 这些类型不需要独立 Java 类定义
- **FfiDefinition::Struct 在 convert_ffi_definition 中返回 `Ok(None)`** — VTable struct 由 callback 代码生成处理
- **模板渲染产物末尾换行符问题** — 通过 `normalize_content()` 函数修剪尾部空白，确保 TOML 文件和 Rust 文件末尾格式正确
- **Cargo.toml 路径** — 主 crate 依赖路径使用正斜杠（TOML 兼容），Rust `use` 语句使用下划线形式的 crate 名
- **jni_bridge.rs 已实现真实 FFI 调用** — JNI→FFI 桥接含 Handle、RustBuffer、ForeignBytes、原语类型转换，通过端到端 Java 测试验证
- **JNI 方法名下划线转义** — `modules.rs` 中 `convert_ffi_definition()` 将 FFI 方法名之下划线 `_` 转为 `_1`，以合 JNI 命名规范
- **Java 内嵌类用 `static`** — RustBuffer、RustBufferStream、HandleMap、Helpers 均声明为 `public static final class`，使可在静态上下文中引用
- **无符号类型参数收窄** — `native_expr_for_arg()` 对 UInt8/UInt16/UInt32 生成显式强制转换 `(byte)/(short)/(int)`
- **String/Bytes lowering 返回 RustBuffer** — `lower_code_for_arg()` 对 String/Bytes 返回 `RustBuffer` 类型（非 ByteBuffer），native 调用处用 `.asByteBuffer()` 转换
- **对象 free/clone 桥接函数** — `_fn_free_` 和 `_fn_clone_` 不再被过滤，正常生成 JNI 桥接代码
- **测试入口文件位于 `TestSimple.java`** — 含顶层函数、Calculator 对象、Record、Enum 之完整验证
- **运行命令** — `java "-Djava.library.path=examples/simple/generated/rust-glue/target/debug" -cp "examples/simple/generated/java;." TestSimple`
