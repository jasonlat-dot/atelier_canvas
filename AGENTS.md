# Atelier 全项目开发续接

本目录是独立的 Atelier 创作软件。界面为 React + TypeScript，业务后端和 Agent 内核为 Rust。

每次开始先读：
1. 根目录 `nextStep.md`：最新完成项、验证、仓库状态及下一步。
2. `docs/PROJECT_DEVELOPMENT.md`：全产品开发文档、模块任务 ID、依赖和验收。
3. `docs/AGENT_PROGRESS.md`：全项目历史进度（沿用文件名）。
4. 涉及 Agent 时另读 `docs/AGENT_REUSE_PLAN.md`。

先核对实际源码、Git 状态和环境，再继续 nextStep 中的下一任务。每完成一步并确认通过后，必须立即更新根目录 nextStep.md，同时记录任务 ID、代码位置、验证结果、未覆盖项和下一步。设计、内核或 Mock 通过不能标成真实用户功能已可用。前后端需要接线时同步修改。

保留各仓库已有未提交修改。当前不打包、不部署生产。本次用户已明确授权完成文档后初始化本项目，并提交/推送到 https://github.com/jasonlat-dot/atelier_canvas.git；保留已有远程历史，不强制覆盖。后续提交/发布范围以用户最新指令为准。

ShellPilot 原仓库仅作来源参考；需要改动前核对其指令和最新进度，避免影响其他工作。文档和脚本用项目相对路径，不依赖本机盘符。源码通过用户仓库同步；数据库、浏览器草稿和密钥不随 Git 自动同步，真实密钥不提交。