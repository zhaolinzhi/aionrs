# aioncore 端 `tool_wire_shape` 透传改造

> **目的**:让 AionUI/AionUI 用户的 provider 设置里能选 `tool_wire_shape = openai_flat_function` 之类的形状,把这个选项透传到 aionrs 进程,解决 `tools[i].name MissingParameter` 错误。
>
> **背景**:aionrs 仓库已经加了 `ToolWireShape::OpenAiFlatFunction` 变体(commit `2426aa7` 及之后)和 `project_openai_flat_function_tool` 投影器,以及能识别 `tools[i].name` 错误的诊断。aioncore 仓库需要把这个字段从 `model_settings` JSON 一路传到 aionrs 进程的 `[providers.xxx.compat]` 段。
>
> **前提**:aioncore `Cargo.toml` 里 `aion-agent` / `aion-config` / `aion-providers` 的 git tag 必须升级到包含 `OpenAiFlatFunction` 变体的版本(目前锁定 `v0.2.8`,需要升到至少包含该变体的 tag,或用 `[patch.crates-io]` 重定向到本地 aionrs 路径)。

---

## 改动清单(共 6 个文件)

| 序号 | 仓库 | 路径 | 改动 |
|------|------|------|------|
| 1 | aioncore | `crates/aionui-ai-agent/src/types.rs` | `AionrsCompatOverrides` 加 `tool_wire_shape` 字段 |
| 2 | aioncore | `crates/aionui-ai-agent/src/factory/aionrs.rs` | `ModelCompatOverrides` 加字段 + 解析 + 合并到 `compat_overrides` |
| 3 | aioncore | `crates/aionui-ai-agent/src/manager/aionrs/agent.rs` | 消费 overrides 写入 `config.compat.tools.tool_wire_shape` |
| 4 | aioncore | `crates/aionui-ai-agent/src/services/provider_health.rs` | 镜像改动 3,health check 也要写 |
| 5 | AionUI | `packages/desktop/src/common/config/storage.ts` | `ModelSettings` 加 `tool_wire_shape` 字段 |
| 6 | AionUI | `packages/desktop/src/renderer/pages/settings/components/AddModelModal.tsx` | 加下拉框 + 状态 + updateModelSettings 调用 |

外加:在 aioncore `Cargo.toml` 把 aionrs 依赖升级到包含修复的 tag(或加 `[patch.crates-io]` 重定向)。

---

## 步骤 0: 升级 aioncore 依赖的 aionrs 版本

aioncore 当前 `Cargo.toml:64-69`:

```toml
aion-agent = { git = "https://github.com/iOfficeAI/aionrs.git", tag = "v0.2.8" }
aion-providers = { git = "https://github.com/iOfficeAI/aionrs.git", tag = "v0.2.8" }
aion-types = { git = "https://github.com/iOfficeAI/aionrs.git", tag = "v0.2.8" }
aion-protocol = { git = "https://github.com/iOfficeAI/aionrs.git", tag = "v0.2.8" }
aion-config = { git = "https://github.com/iOfficeAI/aionrs.git", tag = "v0.2.8" }
aion-mcp = { git = "https://github.com/iOfficeAI/aionrs.git", tag = "v0.2.8" }
```

**做法 A(推荐)**:把 aionrs 仓库 release 一个新 tag(例如 `v0.2.10`),这里 6 行全部改成 `tag = "v0.2.10"`。

**做法 B(开发期)**:在 aioncore 根 `Cargo.toml` 末尾加 `[patch."https://github.com/iOfficeAI/aionrs.git"]`:

```toml
[patch."https://github.com/iOfficeAI/aionrs.git"]
aion-agent = { path = "/Users/zhaolinzhi/Documents/workspaces/2026/aionui/aionrs/crates/aion-agent" }
aion-providers = { path = "/Users/zhaolinzhi/Documents/workspaces/2026/aionui/aionrs/crates/aion-providers" }
aion-types = { path = "/Users/zhaolinzhi/Documents/workspaces/2026/aionui/aionrs/crates/aion-types" }
aion-protocol = { path = "/Users/zhaolinzhi/Documents/workspaces/2026/aionui/aionrs/crates/aion-protocol" }
aion-config = { path = "/Users/zhaolinzhi/Documents/workspaces/2026/aionui/aionrs/crates/aion-config" }
aion-mcp = { path = "/Users/zhaolinzhi/Documents/workspaces/2026/aionui/aionrs/crates/aion-mcp" }
```

---

## 步骤 1: `crates/aionui-ai-agent/src/types.rs`

**目标位置**:`AionrsCompatOverrides` 结构体,大约 line 113-120。

**当前代码**:

```rust
/// Provider-specific compat overrides resolved in the factory.
#[derive(Debug, Clone, Default)]
pub struct AionrsCompatOverrides {
    pub(crate) openai_api_mode: Option<OpenAiApiMode>,
    pub(crate) image_input: Option<ImageInputCapability>,
    pub max_tokens_field: Option<String>,
    pub api_path: Option<String>,
}
```

**改后**:

```rust
use aion_config::compat::ToolWireShape;

/// Provider-specific compat overrides resolved in the factory.
#[derive(Debug, Clone, Default)]
pub struct AionrsCompatOverrides {
    pub(crate) openai_api_mode: Option<OpenAiApiMode>,
    pub(crate) image_input: Option<ImageInputCapability>,
    pub max_tokens_field: Option<String>,
    pub api_path: Option<String>,
    pub tool_wire_shape: Option<ToolWireShape>,  // 新增
}
```

**注意**:`use` 路径按 aioncore 现有 import 风格保持一致(可能已经在文件顶部 import 过 `aion_config`,那就只 import `ToolWireShape`)。

---

## 步骤 2a: `crates/aionui-ai-agent/src/factory/aionrs.rs` — `ModelCompatOverrides`

**目标位置**:`ModelCompatOverrides` 结构体,大约 line 380-384。

**当前代码**:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ModelCompatOverrides {
    pub(crate) image_input: Option<ImageInputCapability>,
    pub(crate) openai_api_mode: Option<OpenAiApiMode>,
}
```

**改后**:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ModelCompatOverrides {
    pub(crate) image_input: Option<ImageInputCapability>,
    pub(crate) openai_api_mode: Option<OpenAiApiMode>,
    pub(crate) tool_wire_shape: Option<ToolWireShape>,  // 新增
}
```

---

## 步骤 2b: 解析 `tool_wire_shape` from `model_settings`

**目标位置**:`resolve_model_compat_overrides` 函数,大约 line 386-410。

**当前代码**(line 397 起):

```rust
Ok(ModelCompatOverrides {
    image_input: settings.image_input.map(|value| match value {
        ModelImageInputCapability::Supported => ImageInputCapability::Supported,
        ModelImageInputCapability::Unsupported => ImageInputCapability::Unsupported,
    }),
    openai_api_mode: settings.openai_api_mode.map(|value| match value {
        ModelOpenAiApiMode::ChatCompletions => OpenAiApiMode::ChatCompletions,
        ModelOpenAiApiMode::Responses => OpenAiApiMode::Responses,
        // ...可能还有 default
    }),
})
```

**改后**:在结构体构造里加 `tool_wire_shape` 字段,并在 `ModelSettings` 类型上加 `tool_wire_shape` 字段定义。

首先,需要给 `ModelSettings`(在同一个文件 `aionrs.rs` 里的类型定义)加字段。**搜索 `ModelSettings`** 看具体定义位置(应该是 aioncore 定义的一个 `pub struct ModelSettings`,带 `image_input`、`openai_api_mode` 等字段)。

在 `ModelSettings` 里加:

```rust
#[serde(rename_all = "snake_case")]
pub enum ModelToolWireShape {
    Native,
    OpenAiFunction,
    OpenAiFlatFunction,
    AnthropicInputSchema,
}

pub struct ModelSettings {
    pub image_input: Option<ModelImageInputCapability>,
    pub openai_api_mode: Option<ModelOpenAiApiMode>,
    pub tool_wire_shape: Option<ModelToolWireShape>,  // 新增
    // ... 其它已有字段
}
```

**反序列化方案**:`ModelSettings` 用 serde 解析 `model_settings` JSON,`tool_wire_shape` 字段在 JSON 里写成 `"openai_flat_function"`,serde 解析时按 `rename_all = "snake_case"` 自动映射到 `OpenAiFlatFunction` 变体。

然后 `ModelCompatOverrides` 构造里加:

```rust
tool_wire_shape: settings.tool_wire_shape.map(|value| match value {
    ModelToolWireShape::Native => ToolWireShape::Native,
    ModelToolWireShape::OpenAiFunction => ToolWireShape::OpenAiFunction,
    ModelToolWireShape::OpenAiFlatFunction => ToolWireShape::OpenAiFlatFunction,
    ModelToolWireShape::AnthropicInputSchema => ToolWireShape::AnthropicInputSchema,
}),
```

---

## 步骤 2c: 把 `tool_wire_shape` 合并到 `compat_overrides`

**目标位置**:`factory/aionrs.rs:102-112`,`resolve_aionrs_url_and_compat_with_mode` 调用之后。

**当前代码**:

```rust
let model_overrides = resolve_model_compat_overrides(&model_id, &row.model_settings)?;
let (base_url, mut compat_overrides) = resolve_aionrs_url_and_compat_with_mode(
    platform, raw_base_url, mapped_provider, model_id, is_full_url, ...,
);
compat_overrides.image_input = model_overrides.image_input;
```

**改后**(在 `compat_overrides.image_input = ...;` 之后加一行):

```rust
compat_overrides.image_input = model_overrides.image_input;
compat_overrides.tool_wire_shape = model_overrides.tool_wire_shape;  // 新增
```

`provider_health.rs:83` 附近有相同模式,**也加这一行**。

---

## 步骤 3: `crates/aionui-ai-agent/src/manager/aionrs/agent.rs`

**目标位置**:大约 line 203-211。

**当前代码**:

```rust
if let Some(mode) = config_extra.compat_overrides.openai_api_mode {
    config.compat.transport.openai_api_mode = Some(mode);
}
if let Some(field) = config_extra.compat_overrides.max_tokens_field {
    config.compat.transport.max_tokens_field = Some(field);
}
if let Some(path) = config_extra.compat_overrides.api_path {
    config.compat.transport.api_path = Some(path);
}
```

**改后**(在末尾加):

```rust
if let Some(mode) = config_extra.compat_overrides.openai_api_mode {
    config.compat.transport.openai_api_mode = Some(mode);
}
if let Some(field) = config_extra.compat_overrides.max_tokens_field {
    config.compat.transport.max_tokens_field = Some(field);
}
if let Some(path) = config_extra.compat_overrides.api_path {
    config.compat.transport.api_path = Some(path);
}
if let Some(shape) = config_extra.compat_overrides.tool_wire_shape {
    config.compat.tools.tool_wire_shape = Some(shape);  // 新增
}
```

---

## 步骤 4: `crates/aionui-ai-agent/src/services/provider_health.rs`

**目标位置**:大约 line 231-240(完整镜像步骤 3)。

**当前代码**:

```rust
if let Some(image_input) = config_extra.compat_overrides.image_input { ... }
if let Some(mode) = config_extra.compat_overrides.openai_api_mode {
    config.compat.transport.openai_api_mode = Some(mode);
}
if let Some(field) = config_extra.compat_overrides.max_tokens_field {
    config.compat.transport.max_tokens_field = Some(field);
}
if let Some(path) = config_extra.compat_overrides.api_path {
    config.compat.transport.api_path = Some(path);
}
```

**改后**(在末尾加):

```rust
if let Some(path) = config_extra.compat_overrides.api_path {
    config.compat.transport.api_path = Some(path);
}
if let Some(shape) = config_extra.compat_overrides.tool_wire_shape {
    config.compat.tools.tool_wire_shape = Some(shape);  // 新增
}
```

---

## 步骤 5: AionUI 前端类型 `packages/desktop/src/common/config/storage.ts`

**目标位置**:`ModelSettings` 类型,大约 line 494-500。

**当前代码**:

```ts
export type ModelSettings = {
  image_input?: ModelImageInputCapability;
  openai_api_mode?: ModelOpenAiApiMode;
  context_window_size?: number;
  max_content_length?: number;
  max_response_length?: number;
};
```

**改后**:

```ts
export type ModelToolWireShape =
  | 'native'
  | 'openai_function'
  | 'openai_flat_function'
  | 'anthropic_input_schema';

export type ModelSettings = {
  image_input?: ModelImageInputCapability;
  openai_api_mode?: ModelOpenAiApiMode;
  tool_wire_shape?: ModelToolWireShape;  // 新增
  context_window_size?: number;
  max_content_length?: number;
  max_response_length?: number;
};
```

**注意**:AionUI 项目里有两份对称的 `ModelSettings`(`lastestui/BadouWorkUi` 和 `lastestui/badouwork`),两处都改。

---

## 步骤 6: AionUI 前端 UI `AddModelModal.tsx`

**目标文件**:
- `packages/desktop/src/renderer/pages/settings/components/AddModelModal.tsx`

**改动**:6 个小步骤。

### 6a. 找 `updateModelSettings` 的实现并加参数

搜 `updateModelSettings` 函数定义(应该在 `packages/desktop/src/common/utils/modelCapabilities.ts` 或类似 utility 文件)。当前签名(参考 line 7):

```ts
updateModelSettings(
  existingSettings: Record<string, ModelSettings> | undefined,
  targetModels: string[],
  imageInput: ModelImageInputCapability,
  openAiApiMode: ModelOpenAiApiMode | 'auto',
  limits: ModelLimitInput,
) => Record<string, ModelSettings>
```

**改后**:

```ts
updateModelSettings(
  existingSettings: Record<string, ModelSettings> | undefined,
  targetModels: string[],
  imageInput: ModelImageInputCapability,
  openAiApiMode: ModelOpenAiApiMode | 'auto',
  toolWireShape: ModelToolWireShape | 'auto',  // 新增
  limits: ModelLimitInput,
) => Record<string, ModelSettings>
```

函数体里构造 `ModelSettings` 时加 `tool_wire_shape` 字段(若非 `'auto'`)。

### 6b. `AddModelModal.tsx` 加状态

在 `useState` 区域(line 65 附近)加:

```ts
const [toolWireShape, setToolWireShape] = useState<ModelToolWireShape | 'auto'>('auto');
```

在 `useEffect`(line 61-81)里加初始化:

```ts
setToolWireShape(settings?.tool_wire_shape ?? 'auto');
```

### 6c. 在表单里加下拉框

参考现有 `openai_api_mode` 的 `<Select>` 写法,加:

```tsx
<div>
  <label>{t('settings.toolWireShape')}</label>
  <Select
    value={toolWireShape}
    onChange={setToolWireShape}
    options={[
      { value: 'native', label: t('settings.toolWireShapeNative') },
      { value: 'openai_function', label: t('settings.toolWireShapeOpenAiFunction') },
      { value: 'openai_flat_function', label: t('settings.toolWireShapeOpenAiFlatFunction') },
      { value: 'anthropic_input_schema', label: t('settings.toolWireShapeAnthropicInputSchema') },
    ]}
  />
</div>
```

### 6d. `handleConfirm` 里传给 `updateModelSettings`

**当前代码**(line 91-101):

```ts
const updatedData: IProvider = {
  ...data,
  models: editingModel ? existingModels : [...existingModels, ...models],
  model_settings: updateModelSettings(
    data.model_settings,
    targetModels,
    imageInput,
    showOpenAiApiMode ? openAiApiMode : 'auto',
    limits
  ),
};
```

**改后**:

```ts
const updatedData: IProvider = {
  ...data,
  models: editingModel ? existingModels : [...existingModels, ...models],
  model_settings: updateModelSettings(
    data.model_settings,
    targetModels,
    imageInput,
    showOpenAiApiMode ? openAiApiMode : 'auto',
    showToolWireShape ? toolWireShape : 'auto',  // 新增(条件:仅 OpenAI 兼容 provider 显示)
    limits
  ),
};
```

### 6e. 加 i18n 翻译键

在 AionUI 的翻译文件(中英文)里加:

```json
"toolWireShape": "Tool wire shape / 工具协议形状",
"toolWireShapeNative": "Native (auto)",
"toolWireShapeOpenAiFunction": "OpenAI function (nested)",
"toolWireShapeOpenAiFlatFunction": "OpenAI flat function (name at top level)",
"toolWireShapeAnthropicInputSchema": "Anthropic input_schema"
```

具体路径按 AionUI 项目实际结构(可能在 `packages/desktop/src/renderer/i18n/` 或 `locales/` 下)。

### 6f. 镜像改动到 `badouwork` 子项目

`lastestui/badouwork/` 下有对称的 `AddModelModal.tsx`,改同样内容。

---

## 步骤 7: 测试

### 7a. aioncore 单元测试

**`crates/aionui-ai-agent/src/factory/aionrs_model_settings_test.rs`**:仿照 `openai_api_mode` 测试加 `tool_wire_shape`:

```rust
#[test]
fn resolves_tool_wire_shape_override() {
    let settings_json = r#"{
        "gpt-4o": { "tool_wire_shape": "openai_flat_function" }
    }"#;
    let overrides = resolve_model_compat_overrides("gpt-4o", settings_json)
        .expect("should parse");
    assert_eq!(overrides.tool_wire_shape, Some(ToolWireShape::OpenAiFlatFunction));
}
```

### 7b. `provider_health.rs` 测试

现有测试 line 423-424 附近加断言:

```rust
assert_eq!(config.compat_overrides.tool_wire_shape, Some(ToolWireShape::OpenAiFlatFunction));
```

### 7c. 端到端验证

启动 BadouWorkUi dev,打开 provider 设置,选 `openai_flat_function`,保存。运行一段对话触发工具调用,检查 aionrs 日志看请求体 `tools[i]` 是平铺形状。

---

## 完整 diff 模板(可复制)

### `crates/aionui-ai-agent/src/types.rs`

```diff
 /// Provider-specific compat overrides resolved in the factory.
 #[derive(Debug, Clone, Default)]
 pub struct AionrsCompatOverrides {
     pub(crate) openai_api_mode: Option<OpenAiApiMode>,
     pub(crate) image_input: Option<ImageInputCapability>,
     pub max_tokens_field: Option<String>,
     pub api_path: Option<String>,
+    pub(crate) tool_wire_shape: Option<ToolWireShape>,  // aion_config::compat::ToolWireShape
 }
```

### `crates/aionui-ai-agent/src/factory/aionrs.rs`

`ModelCompatOverrides` 加字段 + `resolve_model_compat_overrides` 解析 + 调用合并处加一行。具体见步骤 2。

### `crates/aionui-ai-agent/src/manager/aionrs/agent.rs`

```diff
 if let Some(path) = config_extra.compat_overrides.api_path {
     config.compat.transport.api_path = Some(path);
 }
+if let Some(shape) = config_extra.compat_overrides.tool_wire_shape {
+    config.compat.tools.tool_wire_shape = Some(shape);
+}
```

### `crates/aionui-ai-agent/src/services/provider_health.rs`

同上,加 3 行 if let。

### `packages/desktop/src/common/config/storage.ts`

```diff
 export type ModelSettings = {
   image_input?: ModelImageInputCapability;
   openai_api_mode?: ModelOpenAiApiMode;
+  tool_wire_shape?: ModelToolWireShape;
   context_window_size?: number;
   max_content_length?: number;
   max_response_length?: number;
 };

+export type ModelToolWireShape =
+  | 'native'
+  | 'openai_function'
+  | 'openai_flat_function'
+  | 'anthropic_input_schema';
```

### `packages/desktop/src/renderer/pages/settings/components/AddModelModal.tsx`

按步骤 6 改 5 处。

---

## 风险与注意

1. **TypeScript 双重定义**:`lastestui/BadouWorkUi` 和 `lastestui/badouwork` 两个项目共用同一个 `ModelSettings` 类型概念,但物理上**两份文件**。改完一个**别忘了另一个**。
2. **aionrs 依赖升级**:必须先升级到含 `OpenAiFlatFunction` 的 tag,否则 aioncore 编译时 `ToolWireShape` 枚举没有 `OpenAiFlatFunction` 变体而失败。
3. **serde 反序列化**:`ToolWireShape` 在 aionrs 端用 `#[serde(rename = "openai_flat_function")]` 等 snake_case 字符串映射,aioncore 端的 `ModelToolWireShape` 也得用 `rename_all = "snake_case"` 才能反序列化 `model_settings` JSON 里的字符串。
4. **`native` 语义**:AionUI 默认应该是 `'native'`,即不强制覆盖(让 aionrs 自己根据 provider 选)。这与 aionrs 端 `ToolWireShape::Native` 含义一致。
5. **条件显示**:UI 上 `tool_wire_shape` 只在 OpenAI 兼容 provider 上显示才有意义。仿 `showOpenAiApiMode` 加一个 `showToolWireShape` 标志。

---

## 验证步骤(改完后)

1. `cargo build -p aionui-ai-agent`(在 aioncore 仓库)
2. `cargo test -p aionui-ai-agent`
3. 启动 AionUI dev,UI 上看 provider 设置是否多了一个 `Tool wire shape` 下拉框
4. 选 `OpenAI flat function`,保存
5. 看 `aionui-backend.db` 里 `model_settings` 字段是否写入了 `tool_wire_shape: "openai_flat_function"`
6. 跑一段对话,触发工具调用
7. 看 aioncore 日志,确认请求体里 `tools[i]` 形如 `{type:"function", name, description, parameters}`(平铺)
8. 不再出现 `tools[i].name MissingParameter` 错误
