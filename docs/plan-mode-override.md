# Plan Mode 自定义提示词 — 外部调用方接入指南

> 目标读者：通过 `aionrs` CLI 调用 LLM 的外部智能体（例如 AionUI、其他 host 应用、自动化脚本）。
>
> 阅读对象应当熟悉 aionrs 的基本 CLI 用法和 TOML 配置层级（参见 `docs/getting-started.md`、`docs/advanced.md`）。

## 概述

在 aionrs 中，**plan mode** 是一种只读模式，限制智能体只使用 `Read`/`Grep`/`Glob` 工具，先产出实施计划再决定是否动手。模式激活时，aionrs 会把一段 **plan mode instructions** 文本拼到 system prompt 后面，引导模型按 4 阶段流程工作（Understand → Design → Write the plan → Submit for review）。

**本指南介绍的功能**：允许外部调用方完全替换这段内置提示词。所有权归调用方，aionrs 不再强制使用内置文案。

## 行为契约

| 场景 | 注入文本 | 备注 |
|------|---------|------|
| 调用方未传任何 plan mode prompt | 内置默认（`aion_agent::plan::prompt::plan_mode_instructions()`） | 包含 `# Plan Mode` 标题、`ExitPlanMode` 调用指引、`MUST NOT` 写入禁令等 |
| 调用方传入了 prompt | 调用方提供的文本 | 计划模式未激活时**也不会**注入这段文本 |

## 优先级（从高到低）

```
CLI --plan-mode-prompt  >  项目级 .aionrs.toml [plan].prompt  >  全局 ~/.aionrs/config.toml [plan].prompt  >  内置默认
```

后文给出 3 种调用方式，分别对应上述 4 个层级中的前 3 个。

## 方式 1：TOML 配置（推荐用于持久化）

### 1.1 项目级 `.aionrs.toml`

放在 aionrs 启动的当前工作目录下。**只影响当前项目**。

```toml
[plan]
prompt = """
You are in plan mode. Restrict yourself to read-only tools (Read, Grep, Glob).
Think step-by-step. Produce a clear implementation plan in your reply.
When done, call ExitPlanMode to submit for review.
"""
```

### 1.2 全局 `~/.aionrs/config.toml`

放在用户主目录下。**影响所有项目**，但被项目级配置覆盖。

```toml
[plan]
prompt = "Read-only exploration. Plan first. Call ExitPlanMode to submit."
```

### 1.3 多行文本

TOML 支持三引号字符串 `"""..."""`，可以写多行 prompt。注意三引号字符串开头的换行会被忽略，最后一行的换行会保留。

## 方式 2：CLI flag（推荐用于一次性测试）

```bash
aionrs --plan-mode-prompt "Only read files. Plan first. ExitPlanMode after."
```

可以跟其它 flag 组合：

```bash
aionrs \
  --provider openai \
  --plan-mode-prompt "..." \
  --system-prompt "..." \
  --project-dir /path/to/project
```

**注意**：CLI flag 优先级最高，覆盖任何 TOML 设置。

## 方式 3：通过 AionUI / host 集成调用

host 集成（AionUI 等）通过 aionrs 的 JSON stream 协议调用 aionrs。配置路径与 CLI 一致：

- 把 plan mode prompt 写入 `~/.aionrs/config.toml` 的 `[plan].prompt` 字段，或
- 由 host 在调用 aionrs 子进程时通过命令行参数 `--plan-mode-prompt` 注入（如果 host 把 prompt 存在自己的配置里）。

具体怎么落 host 侧的配置由 host 应用决定 —— host 应当让用户能在自己的 UI 上编辑这段文本，然后写到 `~/.aionrs/config.toml` 或运行时拼到 CLI 命令里。

## 验证方法

### 看 plan mode 是否激活

跑一个开启 plan mode 的会话，观察 aionrs 输出的 system prompt（启动时 `--log-level debug` 可以看到完整 system prompt）：

- 如果内置默认生效：system prompt 末尾会包含 `# Plan Mode` 标题、`Read/Grep/Glob`、`MUST NOT`、`ExitPlanMode` 等关键字
- 如果 override 生效：system prompt 末尾只包含你提供的那段文本

### 跑自带的单元测试

```bash
cargo test -p aion-agent --test plan_prompt_file_test
```

会跑 4 个新增的 override 行为测试：

- `override_replaces_default_plan_instructions` —— 传了 override 就用 override，默认不出现
- `none_override_uses_default_plan_instructions` —— 不传就用默认
- `override_does_not_apply_when_plan_mode_inactive` —— plan mode 没激活时 override 也不注入
- `override_change_invalidates_cache` —— override 内容变更会触发缓存重建

## 建议的 prompt 内容

内置默认主要做这几件事，调用方替换时**应当保留核心约束**，否则 plan mode 失去意义：

1. **明确禁止写操作**（编辑、创建、删除文件；运行修改状态的 shell 命令；提交/推送）
2. **明确允许的操作**（Read/Grep/Glob 这类只读工具；写响应文本；调用 `ExitPlanMode`）
3. **建议结构化产出**（Context / Files to modify / Existing code to reuse / Verification）
4. **明确调用 ExitPlanMode** 作为提交方式

最小可行 prompt 示例：

```
You are in plan mode. Use only read-only tools (Read, Grep, Glob).
Do not edit, create, or delete files. Compose your plan as text in your reply,
then call ExitPlanMode to submit it for user review.
```

如果调用方有特殊的 plan 工作流（例如必须先列出待修改文件的列表、必须给出风险评估），可以把对应要求写进 prompt 里。

## 常见问题

### Q: 我设置了 `[plan].prompt` 但启动时 aionrs 没生效？

检查：

1. TOML 语法是否合法（多行字符串结尾的 `"""` 必须独占一行）
2. 文件是否在正确位置（项目级：当前工作目录或 `--project-dir` 指定的目录；全局：`~/.aionrs/config.toml`）
3. 是否被 CLI `--plan-mode-prompt` 覆盖（CLI 优先级更高）

### Q: 我能在 plan mode 激活前就提供 prompt 吗？

可以。`prompt` 是配置项，跟 plan mode 是否激活是独立的两个维度。只有 plan mode 激活时才会注入这段文本。

### Q: 这会影响 token 预算吗？

不会让你预算失控 —— aionrs 在估算 system prompt 的 token 占用时也会按你提供的文本长度算（`engine.rs` 的 `dynamic_system_tokens`）。override 文本越长，预算扣得越多，跟内置默认行为一致。

### Q: 缓存会不会出错？

不会。`SystemPromptCache` 把 `plan_mode_override` 也纳入了失效判断。如果你在会话中途切换 override（理论上当前 aionrs 没有提供运行时切换接口，但配置层面有这个能力），缓存会自动重建。

## 涉及的关键文件

如果外部调用方需要理解实现细节或自行验证：

| 文件 | 角色 |
|------|------|
| `crates/aion-agent/src/plan/prompt.rs` | 内置默认的 prompt 文本 |
| `crates/aion-config/src/plan.rs` | `[plan]` TOML section 的 schema（含 `prompt` 字段） |
| `crates/aion-config/src/config.rs` | `CliArgs` 定义 + 优先级合并逻辑（CLI > TOML > 默认） |
| `crates/aion-cli/src/cli.rs` | `--plan-mode-prompt` CLI flag 定义 |
| `crates/aion-agent/src/engine.rs` | `AgentEngine` 持有 `plan_mode_prompt_override`，三处使用点都走 override |
| `crates/aion-agent/src/context.rs` | `build_system_prompt*` 系列函数接受 `plan_mode_override` 参数；缓存失效判断覆盖 override |
| `docs/advanced.md` | Plan Mode 章节（包含本次新增的 "Customizing the plan mode instructions" 子章节） |

## 给自动化脚本调用方的最小代码示例

如果外部调用方通过命令行直接调用 aionrs 子进程而不是走 JSON stream 协议：

```bash
#!/usr/bin/env bash
set -euo pipefail

# 把 plan mode prompt 写到项目级配置里
cat > .aionrs.toml <<EOF
[plan]
prompt = "Your custom plan mode instructions here"
EOF

# 启动 aionrs 并指定进入 plan mode
# （具体怎么进入 plan mode 由调用方决定 —— 通过提示词引导模型调用 EnterPlanMode，
#  或者 aionrs 后续提供 --plan-mode 直接启动的 flag）
aionrs "$@"
```

如果想完全在运行时注入、不污染配置文件：

```bash
aionrs --plan-mode-prompt "Your custom plan mode instructions here" "$@"
```
