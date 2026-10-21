# 可移植 Agent 核心（R1）

本 crate 从 ShellPilot 提取严格 JSON、工具 Schema/注册校验和上下文预算规划。来源哈希、版本、修改说明和 Apache-2.0 许可在项目根目录 `third_party/shellpilot/`。

它不依赖数据库、HTTP、SSH、Tauri、模型密钥或供应商网络。当前只提供纯逻辑，不包含模型循环、审批执行、摘要生成或长期记忆服务。

- `budget::ModelBudget::plan` 输入由宿主建立的完整历史组和固定/必需/媒体成本，输出预算与压缩建议。`should_compact` 不表示已压缩，`selected_event_ids` 不能直接替代完整历史而宣称语义摘要完成。
- `tools::ToolRegistry` 只接受宿主白名单注册，拒绝未支持的 Schema、重复参数和额外字段。`ValidatedArguments` 是经过格式检查的数据，不是权限或执行许可。
- `strict_json` 保留嵌套重复字段拒绝；Schema 为闭合子集，不是任意 JSON Schema 实现。
- 估算为 UTF-8 字节启发式，不是供应商真实 tokenizer 或用量。ShellPilot Event 分组/估算尚待 R2/R5 的事件适配。

从项目根目录执行：

```powershell
cargo test --offline --manifest-path crates/atelier-agent-core/Cargo.toml
```

宿主通过 Cargo 相对 path 依赖接入。此时仍是 Atelier 内的独立提取副本，ShellPilot 尚未共同依赖该 crate；后续共享化应在接口稳定并回归通过后进行。