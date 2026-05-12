# 项目状态文档 — uniffi-bindgen-java-jni

**日期**: 2026-05-12  
**当前 Phase**: Phase 1（基础设施搭建）→ Phase 2（Pipeline 核心）  
**编译状态**: ❌ 21 errors, 3 warnings（原 23 errors，模板/过滤器问题已解决）

---

## 指导原则

1. **不要进行方案回退** — 坚持 Pipeline 架构（initial IR → general IR → Java IR），不使用 ComponentInterface 旧方案
2. **解决问题优先** — 逐个修复编译错误，不重写整个模块
3. **不要自主做决定** — 遇到需要选择的问题时，先描述问题再询问用户意见

---

## 当前实现进度

| Phase | 步骤 | 状态 | 说明 |
|-------|------|------|------|
| 1 | Step 1: Cargo.toml 依赖 | ✅ 完成 | 所有依赖已配置 |
| 1 | Step 2: 模块目录结构 | ✅ 完成 | 目录和所有源文件已创建 |
| 2 | Step 3: config.rs + context.rs | ✅ 完成 | JavaConfig / Context 基本定义 |
| 2 | Step 4: nodes.rs (Java IR) | ⚠️ 创建但有 bug | Java IR 节点（TypeNode/FfiType/Module 等），模板路径问题 |
| 2 | Step 5: pipeline/mod.rs | ✅ 完成 | general_pipeline() 函数 |
| 2 | Step 6: modules.rs (转换层) | ❌ 编译错误 | general IR → Java IR 转换函数，大量类型不匹配 |
| 3 | Step 7: gen_java/mod.rs | ⚠️ 有警告 | 模板渲染入口，未导入 askama::Template trait |
| 3 | Step 8: gen_rust/mod.rs | ⚠️ 有警告 | Rust 胶水库生成入口，unused variable 警告 |
| — | templates/java/*.java | ⚠️ 文件存在但路径配置错误 | 13 个模板文件已创建，但 Askama 找不到路径 |
| — | templates/rust/*.rs | ⚠️ 文件存在但未集成 | 5 个模板文件已创建，但未通过 Template derive 使用 |
| — | main.rs CLI | ❌ 未实现 | 当前只是 Hello World |

---

## 当前项目依赖

```toml
[dependencies]
anyhow = "1.0"
askama = "0.16.0"
camino = "1.2"
clap = { version = "4.6", features = ["derive"] }
fs-err = "3.3"
heck = "0.5"
indexmap = "2.14"
serde = { version = "1.0", features = ["derive"] }
toml = "1.1"
uniffi_bindgen = { version = "0.31.1", default-features = false }
uniffi_internal_macros = "0.31.1"
uniffi_pipeline = "0.31.1"
```

---

## 编译错误完整清单

### 错误 #1: Askama 模板路径
```
error: template "templates/java/wrapper.java" not found in directories ["D:\\Project\\uniffi-bindgen-java-jna\\templates"]
  --> src\pipeline\nodes.rs:21:53
```
**原因**: `#[template(path = "templates/java/wrapper.java")]` 中 Askama 从 `templates/` 根目录开始查找，所以实际查找路径变成了 `templates/templates/java/wrapper.java`
**修复方案（二选一）**:
- 方案 A: 将 `path` 改为 `"java/wrapper.java"`（不带 templates/ 前缀）
- 方案 B: 在项目根目录创建 `askama.toml` 配置文件指定 `dirs = ["src/templates"]`，然后将 `path` 改为 `"java/wrapper.java"`
```
**原因**: general IR 中 `DefaultValue::Default(TypeNode)` 是元组变体，我们错误地当成了单元变体  
**实际定义**: `pub enum DefaultValue { Default(TypeNode), Literal(LiteralNode) }`  
**修复**: 改为 `general::DefaultValue::Default(ty) => DefaultValue::Default(convert_type_node(ty))`

### 错误 #3: Function 没有 crate_name 字段
```
error[E0609]: no field `crate_name` on type `&general::Function`
  --> src\pipeline\modules.rs:75:76
```
**原因**: `general::Function` 的实际字段中没有 `crate_name`  
**实际字段**: `name, callable, is_async, inputs, return_type, throws, checksum, docstring`  
**修复**: FFI 函数名需要通过 `Callable.ffi_func` 获取，它是 `RustFfiFunctionName` 类型

### 错误 #4: arg.ty 已经是 TypeNode 不是 Type
```
error[E0308]: mismatched types — expected `&Type`, found `&TypeNode`
  --> src\pipeline\modules.rs:85:26
```
**原因**: general IR 中 `Argument.ty` 是 `TypeNode`（包装了 `Type`），不是 `Type`  
**实际定义**: `pub struct Argument { pub ty: TypeNode, ... }`  
**修复**: 需要 1) 更新 `convert_argument` 来接受 `TypeNode` 而不是 `Type`，2) 更新 Java IR 中 `Argument.ty` 类型

### 错误 #5: Argument 没有 by_ref 字段
```
error[E0609]: no field `by_ref` on type `&general::Argument`
  --> src\pipeline\modules.rs:86:21
```
**原因**: general IR 的 `Argument` 没有 `by_ref` 字段  
**实际字段**: `name, ty (TypeNode), optional, default (Option<DefaultValue>)`  
**修复**: 移除我们 Java IR 中 `Argument::by_ref` 字段

### 错误 #6: Type 没有 Object 变体
```
error[E0599]: no variant named `Object` found for enum `general::Type`
   --> src\pipeline\modules.rs:109:24
```
**原因**: general IR 中对象类型的变体名称是 `Interface`，不是 `Object`  
**实际**: `Type::Interface { namespace, name, imp }`  
**修复**: 改为 `general::Type::Interface { namespace, name, imp }`

### 错误 #7: Type 没有 External 变体
```
error[E0599]: no variant named `External` found for enum `general::Type`
   --> src\pipeline\modules.rs:139:24
```
**原因**: general IR 的 `Type` 枚举中不存在 `External` 变体。External 类型在 general IR 中是独立的 `ExternalType` 结构体，通过 `TypeDefinition::External(ExternalType)` 表达  
**修复**: 从 `convert_type` 中移除 External 匹配，External 类型只在 `convert_type_definition` 中处理

### 错误 #8: DefaultValue::Literal 包装类型不对
```
error[E0308]: mismatched types — expected `&Literal`, found `&LiteralNode`
   --> src\pipeline\modules.rs:150:86
```
**原因**: general IR 中 `DefaultValue::Literal(LiteralNode)` 是 `LiteralNode` 包装，inner 才是 `Literal`  
**实际**: `LiteralNode { lit: Literal }` 通过 `#[node(wraps)]` 可以自动解引用  
**修复**: 通过 `.lit` 访问或使用自动解引用

### 错误 #9-10: convert_type 接受 TypeNode 而非 Type（2 处）
```
error[E0308]: mismatched types — expected `&Type`, found `&TypeNode`
   --> src\pipeline\modules.rs:231:42 (Field.ty)
   --> src\pipeline\modules.rs:255:54 (另一个 Field.ty)
```
**原因**: `Field.ty` 和 `VTableMethod` 使用的字段是 `TypeNode` 不是 `Type`  
**修复**: 创建一个 `convert_type_node()` 函数，从 TypeNode 的 `.ty` 取出 inner Type 后转换

### 错误 #11: Enum 没有 is_flat() 方法
```
error[E0599]: no method named `is_flat` found for reference `&Enum`
   --> src\pipeline\modules.rs:264:24
```
**原因**: `is_flat` 是字段不是方法  
**实际**: `pub is_flat: bool`  
**修复**: `e.is_flat` 不加括号

### 错误 #12: FfiDefinition 没有 CallbackFunction 变体
```
error[E0599]: no variant named `CallbackFunction` found for enum `FfiDefinition`
   --> src\pipeline\modules.rs:314:33
```
**原因**: general IR 中对应变体名称是 `FunctionType`，不是 `CallbackFunction`  
**实际**: `FfiDefinition::FunctionType(FfiFunctionType)`  
**修复**: 改为 `general::FfiDefinition::FunctionType(ft)`

### 错误 #13: RustFfiFunctionName 不实现 Display
```
error[E0277]: `RustFfiFunctionName` doesn't implement `std::fmt::Display`
   --> src\pipeline\modules.rs:336:17
```
**原因**: 用 `{}` 格式化，但只有 `.0` 字段是 String  
**修复**: 使用 `func.name.0` 访问内部 String

### 错误 #14: RustFfiFunctionName 不能直接赋值给 String
```
error[E0308]: mismatched types — expected `String`, found `RustFfiFunctionName`
   --> src\pipeline\modules.rs:339:23
```
**原因**: `ffi.name` 是 `RustFfiFunctionName` 类型，但 Java IR 中的 `FfiFunction.name` 是 `String`  
**修复**: 1) 改为 `ffi.name.0.clone()` 或 2) 将 Java IR 的 FfiFunction.name 改为 `RustFfiFunctionName`

### 错误 #15-17: FfiArgument.ty 已经是 FfiTypeNode
```
error[E0308]: mismatched types — expected `&FfiType`, found `&FfiTypeNode`
   --> src\pipeline\modules.rs:347:50 (FfiArgument)
   --> src\pipeline\modules.rs:378 (FfiStruct)
   --> src\pipeline\modules.rs:385 (FfiReturnType)
```
**原因**: general IR 中所有 Ffi 相关字段都用 `FfiTypeNode` 包装，不是裸 `FfiType`  
**实际**: `FfiArgument { ty: FfiTypeNode }`, `FfiField { ty: FfiTypeNode }`, `FfiReturnType { ty: Option<FfiTypeNode> }`  
**修复**: 1) 创建 `convert_ffi_type_node()` 来从 `FfiTypeNode.ty` 提取 2) 或直接存 `FfiTypeNode`

### 错误 #18: FfiReturnType 不是 Option
```
error[E0599]: no method named `as_ref` found for struct `FfiReturnType`
   --> src\pipeline\modules.rs:351:47
```
**原因**: general IR 的 `FfiReturnType` 是 struct `{ pub ty: Option<FfiTypeNode> }` 不是 Option  
**修复**: 使用 `.ty.as_ref()`

### 错误 #19-20: FfiType 没有 Boolean/String 变体
```
error[E0599]: no variant named `Boolean` found for enum `general::FfiType`
error[E0599]: no variant named `String` found for enum `general::FfiType`
```
**原因**: general IR 的 `FfiType` Enum 不包含 Boolean 和 String（它们通过 RustBuffer 传递）  
**修复**: 在 convert_ffi_type 中移除这些变体的匹配

### 错误 #21: FfiType 没有 Bytes 变体
```
error[E0599]: no variant named `Bytes` found for enum `general::FfiType`
```
**原因**: 同上，Bytes 通过 RustBuffer/ForeignBytes 传递  
**修复**: 移除 Bytes 匹配

### 错误 #22: FfiType 没有 RustArc 变体
```
error[E0599]: no variant named `RustArc` found for enum `general::FfiType`
```
**原因**: 对象引用通过 `Handle` 传递  
**修复**: 移除 RustArc 匹配

### 错误 #23: FfiType::Struct 参数是 FfiStructName 不是 String
```
error[E0308]: mismatched types — expected `String`, found `FfiStructName`
   --> src\pipeline\modules.rs:378:59
```
**原因**: general IR 中 `FfiType::Struct(FfiStructName)` 包装了名称  
**修复**: 使用 `name.0.clone()` 访问内部 String

### 错误 #24: Template trait 未导入
```
error[E0599]: no method named `render` found for reference `&Module`
  --> src\gen_java\mod.rs:37:31
```
**原因**: 缺少 `use askama::Template;`  
**修复**: 添加 import

---

## general IR 实际结构速查

### TypeDefinition 变体
```rust
TypeDefinition
├── Interface(Interface)        // 对象/trait（注意变体名是 Interface 不是 Object）
├── CallbackInterface(CallbackInterface)
├── Record(Record)
├── Enum(Enum)
├── Custom(CustomType)
├── Simple(TypeNode)            // 简单包装类型
├── Optional(OptionalType)
├── Sequence(SequenceType)
├── Map(MapType)
└── External(ExternalType)      // 跨 crate 外部类型
```

### FfiDefinition 变体
```rust
FfiDefinition
├── RustFunction(FfiFunction)         // 顶层 FFI 函数
├── FunctionType(FfiFunctionType)     // 回调函数类型（注意不是 CallbackFunction）
└── Struct(FfiStruct)                // FFI 结构体（如 VTable）
```

### FfiType 变体（仅这些！）
```
UInt8 | Int8 | UInt16 | Int16 | UInt32 | Int32 | UInt64 | Int64
Float32 | Float64 | RustBuffer(Option<String>) | ForeignBytes
Function(FfiFunctionTypeName) | Struct(FfiStructName) | Handle(HandleKind)
RustCallStatus | Reference(Box<FfiType>) | MutReference(Box<FfiType>) | VoidPointer
```
- ❌ 没有 `Boolean`、`String`、`Bytes`、`RustArc`

### Type 变体
```
UInt8 | Int8 | ... | Float32 | Float64 | Boolean | String | Bytes
Timestamp | Duration
Optional { inner_type: Box<Type> }
Sequence { inner_type: Box<Type> }
Map { key_type: Box<Type>, value_type: Box<Type> }
Interface { namespace, name, imp }     ← 变体名是 Interface 不是 Object
Record { namespace, name }
Enum { namespace, name }
CallbackInterface { namespace, name }
Custom { namespace, name, builtin: Box<Type> }
```
- ❌ 没有 `External` 变体 — External 类型通过 `TypeDefinition::External` 表达

### DefaultValue / Literal
```rust
DefaultValue::Default(TypeNode)    // 元组变体，包含类型信息
DefaultValue::Literal(LiteralNode) // 包装类型
LiteralNode { lit: Literal }      // #[node(wraps)] 可自动解引用
```

### FfiReturnType
```rust
struct FfiReturnType { pub ty: Option<FfiTypeNode> }  // struct 不是 Option
```

---

## 待创建的文件清单

### 模板文件 (templates/java/)
- [ ] wrapper.java
- [ ] Function.java
- [ ] Object.java
- [ ] Interface.java
- [ ] Record.java
- [ ] Enum.java
- [ ] CallbackInterface.java
- [ ] CallbackInterfaceImpl.java
- [ ] FfiConverter.java
- [ ] RustBuffer.java
- [ ] RustBufferStream.java
- [ ] HandleMap.java
- [ ] Helpers.java

### 模板文件 (templates/rust/)
- [ ] cargo_toml.rs
- [ ] lib.rs
- [ ] jni_bridge.rs
- [ ] jni_types.rs
- [ ] jni_callback.rs

### 源码文件 (gen_java/)
- [x] mod.rs（需修复）
- [x] primitives.rs
- [x] compounds.rs
- [x] object.rs
- [x] callback_interface.rs
- [x] enum_.rs
- [x] record.rs
- [x] custom.rs
- [x] miscellany.rs

### 源码文件 (gen_rust/)
- [x] mod.rs（需修复）
- [x] jni_func.rs
- [x] jni_types.rs
- [x] callback_gen.rs
- [x] cargo_toml.rs

### 源码文件 (pipeline/)
- [x] mod.rs
- [x] config.rs
- [x] context.rs
- [x] nodes.rs（需修复 — 模板路径）
- [x] modules.rs（需大规模修复 — 23 个错误）
- [x] types.rs
- [x] callables.rs
- [x] enums.rs
- [x] records.rs
- [x] interfaces.rs
- [x] callback_interfaces.rs
- [x] default.rs
- [x] jni_signature.rs

### 根文件
- [x] Cargo.toml
- [x] src/lib.rs
- [ ] src/main.rs（CLI — 尚未实现，当前 Hello World）
