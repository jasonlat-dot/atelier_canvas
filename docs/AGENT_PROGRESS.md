# Atelier 全项目开发进度

更新时间：2026-10-22。以实际源码和本文件的实测结果续接；原 README、03/04 文档中的“本环境”描述属于原型交付时的历史记录。


## 最新实施快照（2026-10-22，以本节覆盖后续历史状态）

当前 S1 实现文本供应商/模型管理。代码位置：`services/api/src/providers.rs`、`credentials.rs`、`migrations/202610220001_text_providers.sql`、`apps/web/src/features/providers/`、`lib/providerApi.ts`。

- [x] 已接线：管理员新增/编辑/启停供应商，AES-256-GCM 保存凭据（provider ID 绑定、随机 nonce、不得回传）；密钥保留/替换/移除与改目的地址保护，revision 冲突及管理审计。
- [x] 已接线：已保存配置的只读模型目录探测，10 秒超时、响应体上限、重定向拒绝、上游错误脱敏；文本模型设置/启停与用户可见目录。图片视频 Mock 目录保留独立。
- [x] 前端接线：管理表单、获取上游模型、测试连接状态；普通对话页读取已开放文本模型，发送仍禁用，S2 尚未开始。Mock 模式不保存/发送供应商密钥。
- [x] 新增启动脚本 `scripts/start-dev-api.ps1`，可使用环境密钥或忽略的 `.atelier/credential.key`；现有 Rust 启动批处理改为复用脚本。说明：`docs/S1_PROVIDER_SETUP.md`。
- [x] 首轮与补充 PostgreSQL 集成通过：独立实例/干净库、真实迁移和生产路由、权限、密文/保留/清除、重开持久化、配置冲突、目录过滤、错误分类；`scripts/test-providers.ps1` 正常关闭本任务临时实例。原 PostgreSQL 服务未启动或修改。
- [x] 最终服务端验证：普通单元/回环 8 项通过，默认忽略的数据库生命周期另行执行 1 项通过；包含真实 10 秒超时、Origin 拒绝、无加密密钥不落明文。严格 Clippy（all-targets/-D warnings）通过。前端 16 个 TS/TSX 转译、Mock/HTTP 含供应商回归和类型检查通过；随后已更新 nextStep。
- [ ] 浏览器 Target.createTarget 派发错误仍存在，真实供应商/对话未测试；不能称 S1 真实用户验收或 S2 已完成。当前不打包。

## 最新续接快照（2026-10-21，以此覆盖下方历史状态）

最新用户要求：文档覆盖整个软件，包括 Google 注册登录、积分/支付/订阅、画布、素材、生成、Agent 和所有后台；每一步确认通过后更新根目录 nextStep.md；本次完成后初始化并推送用户指定仓库。入口：[总开发文档](PROJECT_DEVELOPMENT.md)、[下一步](../nextStep.md)。

- [x] R0/R1 限定范围已完成：独立 Rust 参数校验/预算内核、能力接口和前端状态接线；核心 16 项/API 2 项测试通过，前端回归与类型检查、Rust check 通过。不是完整 Agent 用户功能完成。
- [x] `scripts/verify-agent.ps1` 已实际运行通过；4 个 ShellPilot 来源文件哈希一致。源码来源与许可在 third_party/shellpilot；未修改来源仓库。
- [x] 新增 `docs/PROJECT_DEVELOPMENT.md` 全项目总文档，包含功能清单、现状、数据/API 设计、里程碑、待决策项和验收；Agent 清单保留为子计划。文档最终核验与 nextStep 见下方本轮记录。
- [ ] PostgreSQL、真实 Google/图片/视频/支付未联调；浏览器 Target.createTarget 错误仍使视觉与交互验收未完成，不能标通过。
- [x] 本次 Git 已初始化 main，初始提交 4b36ada 已成功推送至用户仓库，并核对远程 main SHA 一致；最新文档回执见 nextStep。
- 最新用户调整顺序：S1 管理端供应商/文本模型 → S2 普通对话（不压缩/不工具）→ S3 历史与普通对话压缩 → S4 画布/媒体/创作工具 → S5 Google 注册登录 → S6 积分奖励 → S7 支付订阅/后台 → S8 上线准备。必要数据库测试随对应功能做，不先展开整套钱包联调。
## 项目定位与范围

- 独立项目：`E:/project/ShellPilot/Atelier_React_Rust_Starter`。React + TypeScript + React Flow 创作前端；Rust Axum + SQLx + PostgreSQL API。
- 保留现有浅色界面与单一前端的用户/管理员布局。本轮未修改其他 ShellPilot 仓库。
- 接手时目录不是 Git 仓库，已有 `AppErrorBoundary.tsx`、npm 依赖、环境配置及 Cargo.lock；保持原样，不重置任何既有文件。
- 当前不打包、不生产部署；本次用户已授权初始化提交与推送。没有接入真实模型、Google 登录、支付或 S3。

## 历史记录：首次开发基线及 HTTP 适配修复

- [x] 核对原型交付文档、前端入口、画布、Mock 状态、Rust handlers 和 SQL 迁移。
  - 位置：`README.md`、`docs/03_本轮工程实现与迁移说明.md`、`docs/04_API与安全验收清单.md`、`apps/web/src/`、`services/api/src/main.rs`。
  - 结论：现有产品为 Mock 创作流程；连线不是工作流执行引擎，开发登录不是正式认证。
- [x] 修复测试写死 Linux TypeScript 安装路径，改为从本项目前端依赖解析；显式使用 Node webcrypto，适配当前 Windows Node 18 环境。
  - 位置：`tests/verify_source.cjs`；统一命令：`apps/web/package.json` 的 `npm test`。
  - 验证：12 个 TS/TSX 文件转译语法检查及既有扣费、幂等、退款、版本冲突、隔离和管理员模拟校验通过。
- [x] 修复 HTTP 写请求自定义请求头覆盖默认 JSON Content-Type 的问题。
  - 位置：`apps/web/src/lib/api.ts::request`。
  - 验证：`tests/verify_http.cjs` 使用真实 API 适配代码和模拟 fetch，检查画布 If-Match、任务 Idempotency-Key、JSON Content-Type、Cookie credentials 和 409 错误透传，通过。
- [x] 接通 HTTP 钱包流水到前端状态，按当前服务端会话补充流水归属；刷新身份时清空旧流水与 Mock 签到状态。
  - 位置：`apps/web/src/lib/api.ts::refreshServerView`、`apps/web/src/lib/types.ts::Wallet`、`apps/web/src/lib/mockDb.ts::hydrateRemote`。
  - 验证：`tests/verify_http.cjs` 检查余额、流水归属、换身份清理和 HTTP 快照不覆盖本地演示数据，通过。
- [x] 移除缓存任务响应时的本地余额扣减，以 `/wallet` 返回余额为准；重复响应只更新同一个任务。
  - 位置：`apps/web/src/lib/mockDb.ts::cacheRemoteJob`。
  - 验证：`tests/verify_http.cjs` 重复缓存同一任务后只有一条记录，余额不重复扣减，通过。界面余额在下一次服务端轮询时更新。
- [x] Windows 开发机实测 `npm --prefix apps/web test`、`npm --prefix apps/web run typecheck`，通过。
- [x] 实测 `cargo check --offline`，通过；`cargo test --offline` 成功编译并执行，但现有 Rust 测试数量为 **0**，不能记为业务测试通过。
  - 位置：`services/api/Cargo.toml`、`services/api/src/main.rs`。
  - 仍有既有 unused imports 警告：`put`、`Serialize`。未改 Rust 源码。
- [x] 既有 Vite 服务 `http://127.0.0.1:5173/` 返回 200，标题为 Atelier；`/src/lib/api.ts` 转换请求返回 200。
  - 仅证明开发服务器可提供页面和模块，不证明浏览器渲染或交互通过。

## 未完成／验证边界

- [ ] M1 浏览器 UI 验收：通过 tabbit 技能创建任务后，创建页面时报 `Target.createTarget: Task-scoped CDP command could not be dispatched`；诊断显示任务无已拥有页面。未完成视觉、画布交互和刷新恢复验收。未执行前端生产构建（当前不打包）。
- [ ] M2 PostgreSQL 联调：PATH 中未找到 Docker / psql；未启动数据库、执行真实迁移或 HTTP 全链路。HTTP 新增测试为模拟 fetch，不是 Rust 集成测试。
- [ ] M2 增加真实 Rust 业务测试：同键并发、取消/完成竞争、积分不足回滚、跨用户访问和旧版本保存。
- [ ] 后续核实并修复：`count > 1` 目前 Mock 与 Rust Worker 都只生成一个资产，画布也只读取第一个结果；不能将批量生成视为已实现。
- [ ] 后续核实并修复：画布离开页面前的 debounce 保存、保存冲突恢复和多标签页同步。
- [ ] 真实认证、媒体持久化、供应商接入、队列与正式账本，按里程碑逐项接通，完成一项再更新标记与验证结果。

## 历史基线待办（最新优先级见 nextStep）

1. 完成浏览器画布回归，并优先修复用户编辑内容丢失风险。
2. 在本机 PostgreSQL 环境跑通开发登录 → 创建项目 → 保存/刷新 → 提交/取消任务 → 钱包流水，增加 Rust 数据库集成测试。
3. 修复多结果回流，再推进正式登录、资产存储与真实图片生成。

## 复验命令（无需发行打包）

```powershell
# 项目根目录
npm --prefix apps/web test
npm --prefix apps/web run typecheck
# services/api 目录
cargo check --offline
cargo test --offline
```
## 全项目文档与仓库初始化（2026-10-21）

- [x] PLAN-01/02/03：总开发文档含 125 个任务项；19 项模块覆盖、任务 ID 和本地链接核验通过，根目录 nextStep.md 已在验证后写入，AGENTS/README/Agent 子计划已统一入口。
- [x] 提交前重新运行 scripts/verify-agent.ps1：前端回归/类型检查、核心 16 项/API 2 项和 cargo check 均通过。数据库与浏览器仍未验收。
- [x] Git 已初始化 main，origin 指向用户仓库；确认远程无 refs，已检查不提交真实环境配置、凭据或生成产物。提交/推送成功回执后补记。

- [x] REPO-01/02 完成：初始提交 4b36ada 已推送 origin/main，远程 SHA 与本地基线一致；随后同步本次交接回执。来源仓库未改；无打包或生产部署。该条历史安排已由最新供应商优先计划覆盖。

## ORDER-01：用户调整开发顺序（2026-10-21）

- 本步修改总计划 v1.1 的优先顺序和依赖，Agent R 编号保持稳定但基础 R3 先于 R2/R5；注册、积分及商业化后置。
- S1 明确最小供应商/凭据管理、文本模型开放与测试连接，必要后端管理权限沿用本机开发身份；S2 只做普通对话，S3 再接压缩，S4 接画布。
- 仅修改文档；原核心与前端/Rust 测试结果保留为历史，不当成本次重跑。文档验证通过后写入 nextStep。

- [x] ORDER-01 已核验：S1→S7 顺序、Agent 子任务关联一致；保留全部 125 个唯一功能任务，文档本地链接和 diff 格式通过；随后已更新根目录 nextStep.md。只改文档，无新增业务完成项。

- S1 交接：前后端/数据库子范围已验证，S1 浏览器及真实供应商验收仍未通过，S2 发送保持关闭。UI 测试端口 5174 被已有服务占用，未停止该服务；Tabbit 建页也失败。不得据此宣称真实 UI 可用。详细启动、协议和密钥恢复见 docs/S1_PROVIDER_SETUP.md。

- [x] S1 实现提交 9b56b14 已推送并核对远程 main SHA，原 PostgreSQL 服务保持 Stopped；本次无打包/真实供应商调用。交接回执追加至 nextStep 并同步。
