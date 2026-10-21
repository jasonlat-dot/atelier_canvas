# Atelier 下一步与跨会话续接

更新：2026-10-21。每完成一步并确认通过后必须更新本文件；先核对实际 Git/源码再继续，不能仅依赖聊天。

## 当前开发优先顺序（最新用户指令）

管理端供应商/模型接入 → 普通对话测试（不压缩、不工具）→ 历史与普通对话上下文压缩 → 画布工作台/创作工具 → Google 注册登录 → 积分/签到/奖励 → 支付/订阅及其余功能。

这个顺序覆盖旧版先注册/钱包的安排；前期使用本机 DEV_AUTH 开发身份和最小后端管理员授权，基础对话与画布开发不依赖正式 Google 或积分钱包。

## 最新用户要求

- 开发文档覆盖整个 Atelier：Google 登录/注册、用户权限、画布、素材、图片/视频生成、Agent/工具审批/上下文压缩/长期记忆、积分/签到/支付/订阅、后台与部署运维。
- 文档先落盘再逐步实现；每步通过后更新 nextStep.md。
- 完成本轮后初始化并提交/推送到 `https://github.com/jasonlat-dot/atelier_canvas.git`。
- 不打包、不部署生产；保护既有修改，不覆盖 ShellPilot 来源仓库的工作。

## 已完成并验证的步骤

### R0/R1：Agent 第一批基础（前一步实测）

- `crates/atelier-agent-core/` 提取严格 JSON、工具参数 Schema/白名单与上下文预算；16 项 Rust 测试通过。
- `services/api/src/agent.rs` 增加认证后的 `/api/v1/agent/capabilities`；2 项宿主测试通过，核心能力与用户功能分开。
- `apps/web/src/features/chat/ChatPage.tsx`、`types.ts`、`lib/api.ts` 接能力状态，移除虚构 token/已完成能力文案；HTTP 模式未接入真实对话时禁用发送，Mock 保留明确演示。
- `tests/verify_http.cjs` 检查 HTTP/Mock 能力、401 不回退伪造状态及既有保存/任务/钱包；前端回归和类型检查通过。
- `scripts/verify-agent.ps1` 实际运行通过；核心/API 格式检查和 4 个来源 SHA256 核对通过。
- 这不是完整 Agent 完成：模型对话、审批执行、摘要压缩、长期记忆服务均未接入。

### PLAN-01/02：完整项目开发文档与入口（本步核验）

- [总开发文档](docs/PROJECT_DEVELOPMENT.md)：覆盖全部产品模块、当前状态、数据/API 设计、S0–S8 依赖顺序、待决策项和验收；包括旧原型的新 React 尚缺的积分策略/系统设置页。
- [工作规则](AGENTS.md)、[README](README.md)、[项目进度](docs/AGENT_PROGRESS.md)、[Agent 子计划](docs/AGENT_REUSE_PLAN.md) 已统一入口，并补记中断前 R1 的实际通过结果。
- 文档模块覆盖检查、唯一任务 ID 检查、已有本地链接检查通过；下一步文档已按用户要求在核验后写入本文件。最终链接和提交前检查另列实际回执。

## 仓库状态

- 目标：`https://github.com/jasonlat-dot/atelier_canvas.git`。
- 初始化前远程为空；推送后已创建 main，配置为跟踪 origin/main。
- 初始基线提交 `4b36ada` 已成功推送；已用 git ls-remote 验证远程 main 与本地基线完整 SHA 一致。本次交接回执作为后续文档提交同步，最新提交以 git log / origin/main 核对，不依赖自引用哈希。

## 未完成与环境限制

- 浏览器 Tabbit 创建页面仍报 `Target.createTarget: Task-scoped CDP command could not be dispatched`；已结束任务。仅开发模块 200，未验收渲染/交互。
- 尚未真实连接 PostgreSQL，也未验收 Google、S3、供应商、支付。PATH 中此前未找到 Docker/psql；需核对新电脑环境。
- 当前仍有画布 debounce 离开丢保存风险、revision 冲突恢复、多标签页同步及 count>1 只生成/展示一个资产的问题。
- 具体 Google 邮箱范围、供应商、支付渠道、套餐/订阅、奖励/文本收费等见总文档 DEC-01–08；尚未确定的价格均不能用原型数字直接上线。

## 下一项与通过标准

1. **本步 ORDER-01 已完成并验证。** 总计划 v1.1、Agent 子计划、README/AGENTS/项目进度已统一顺序；原 125 个功能任务保留，没有把任何新业务标为已实现。本步只改文档，不重跑无关运行时测试。
2. **下一开发阶段：S1 管理端供应商/文本模型接入。** 入口：`apps/web/src/features/admin/AdminPage.tsx`、`services/api/src/main.rs`、`services/api/migrations/`；拟拆分 providers/catalog 的持久化与管理接口。参考 ShellPilot provider 模块和来源记录，保护源仓库修改。
3. **S1 的最小交付：** 管理员新增/编辑/启停一个文本供应商、保存服务端凭据、测试连接、配置并开放文本模型；普通用户可读开放目录。API 必须校验后端管理员角色，密钥不回传，保存/重启后仍可读取配置。
4. **必要环境与验收：** 一次性 PostgreSQL 的供应商/模型迁移、管理权限、凭据掩码、停用/可见性、连接失败/超时测试；先回环供应商，再记录真实连接结果。数据库是该链路支撑，不先展开生成/钱包全量回归。
5. **S2 然后做普通对话：** 使用 S1 开放模型，先单轮回复，再流式、停止、有界多轮、空模型/上游错误。没有压缩、工具、审批、记忆、画布或积分扣费；超限明确提示，不静默删除历史。
6. **S3/S4 再增强：** S3 补持久历史/事件与普通对话压缩（R2→R5）；S4 做画布编辑保存、创作工具/审批、素材和图片视频，再补长期记忆；画布保存和多产物缺陷在对应启用前解决。
7. **S5/S6 之后：** Google 注册登录、账户权限完善，再接正式积分钱包、注册/签到奖励；充值/支付/订阅与剩余运营接着做。公开上线前仍须完整认证/计费验收。

## 复验与继续指令

```powershell
# 项目根目录，不打包；首次安装依赖见总文档
./scripts/verify-agent.ps1
```

新会话可直接使用：

> 继续 Atelier 项目。先读根目录 AGENTS.md、nextStep.md、docs/PROJECT_DEVELOPMENT.md、docs/AGENT_PROGRESS.md，核对 Git/源码/环境，再按 nextStep 的下一项工作；涉及 Agent 再读子计划。保留已有修改，每完成一步并确认通过后更新 nextStep.md，不打包、不生产部署。

Git 同步源码和文档，不同步数据库、浏览器草稿或密钥。换电脑必须克隆/拉取完整项目并重建本地配置；来源和锁文件已随项目保留。
## 本步提交前复验回执

- 本次重新执行 scripts/verify-agent.ps1：前端回归、13 个 TS/TSX 转译及类型检查通过；独立核心 16 项、API 2 项和 cargo check 通过。
- 总文档 125 个唯一任务项、19 项模块覆盖检查通过；所有已跟踪 Markdown 的本地链接与凭据模式扫描通过。
- .env、node_modules、target 与本地产物均未跟踪；仅提交示例环境配置。已添加 .gitattributes 统一跨电脑文本换行。

## 仓库同步成功回执

- 远程：https://github.com/jasonlat-dot/atelier_canvas.git ，分支 main。
- 首次推送：4b36ada（chore: initialize Atelier with Rust agent core and full development plan），Git 返回 new branch main -> main。
- 推送后远程 refs/heads/main 与本地初始提交 SHA 核对一致；首次禁用交互认证失败，随后正常 Git 凭据助手认证推送成功，无待用户登录事项。
- 后续起点已按最新用户指令改为 S1 管理端供应商/文本模型接入；不要照旧的 BASE-03→注册/钱包顺序推进。执行前核对 Git 并读取最新总计划。

## ORDER-01 本步验证回执（2026-10-21）

- 已检查 S1→S7 顺序与 Agent R3→R2/R5→R4 关联一致。125 个功能任务 ID 保留且唯一，已有文档本地链接通过，git diff --check 通过。
- 本次仅文档调整，未修改业务代码、迁移或测试；先前 16+2 项 Rust 与前端检查仍是历史结果。
- 完成本步核验后更新本文件，下一项明确为供应商管理；本次顺序变更单独提交同步，最新提交与远程状态通过 Git 核对。
