# Atelier 全项目开发进度

更新时间：2026-10-21。以实际源码和本文件的实测结果续接；原 README、03/04 文档中的“本环境”描述属于原型交付时的历史记录。


## 最新续接快照（2026-10-21，以此覆盖下方历史状态）

最新用户要求：文档覆盖整个软件，包括 Google 注册登录、积分/支付/订阅、画布、素材、生成、Agent 和所有后台；每一步确认通过后更新根目录 nextStep.md；本次完成后初始化并推送用户指定仓库。入口：[总开发文档](PROJECT_DEVELOPMENT.md)、[下一步](../nextStep.md)。

- [x] R0/R1 限定范围已完成：独立 Rust 参数校验/预算内核、能力接口和前端状态接线；核心 16 项/API 2 项测试通过，前端回归与类型检查、Rust check 通过。不是完整 Agent 用户功能完成。
- [x] `scripts/verify-agent.ps1` 已实际运行通过；4 个 ShellPilot 来源文件哈希一致。源码来源与许可在 third_party/shellpilot；未修改来源仓库。
- [x] 新增 `docs/PROJECT_DEVELOPMENT.md` 全项目总文档，包含功能清单、现状、数据/API 设计、里程碑、待决策项和验收；Agent 清单保留为子计划。文档最终核验与 nextStep 见下方本轮记录。
- [ ] PostgreSQL、真实 Google/图片/视频/支付未联调；浏览器 Target.createTarget 错误仍使视觉与交互验收未完成，不能标通过。
- [ ] 本次 Git 初始化/提交/推送执行情况以根目录 nextStep 及实际 Git 回执为准，尚未核验前不称已同步。
- 下一阶段按总计划 S1：优先建立一次性 PostgreSQL 集成基线，随后处理现有画布保存与批量产物问题；Agent R2 仍是后续子任务。
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
