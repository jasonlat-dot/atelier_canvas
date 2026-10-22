# Atelier 下一步与跨会话续接

更新：2026-10-22。本文件在每一步确认通过后更新；先核对源码/Git/环境，再按下一项继续。

## 用户要求与当前顺序

管理端供应商/模型接入 → 普通对话（先不接压缩/工具）→ 历史与普通对话压缩 → 画布工作台/创作工具 → Google 注册登录 → 积分奖励 → 支付订阅及其余功能。

前期仅本机 DEV_AUTH 开发身份，后端最小管理员权限仍须验证。保留既有修改，不打包、不部署生产；源代码/文档按用户授权同步 `https://github.com/jasonlat-dot/atelier_canvas.git`。每步核验后更新本文件，不能只在聊天交接。

## 最新完成：S1 首个文本渠道（2026-10-22）

### 已实现且有测试的范围

- `services/api/migrations/202610220001_text_providers.sql`：供应商、文本模型、最小管理审计；真实 PostgreSQL 迁移通过，不改旧迁移。
- `services/api/src/providers.rs`：后端管理员检查、供应商新增/编辑/启停、revision、模型配置/开放、普通用户文本目录。供应商停用或模型关闭后从用户目录隐藏；图片/视频 Mock 目录独立保留。
- `services/api/src/credentials.rs`：AES-256-GCM、随机 nonce、供应商 ID 绑定；不回传/打印密钥，缺少加密配置拒绝保存，无明文降级。支持保留/替换/移除；变更目的地址需显式重新提供或清除凭据。
- 只读连接测试：Chat Completions 兼容渠道的 GET /models；10 秒总超时、响应体上限、重定向拒绝、失败分类和脱敏，不发送生成请求。
- `apps/web/src/features/providers/ProviderConsole.tsx`、`types.ts`、`lib/providerApi.ts`：管理表单、获取上游模型、测试连接与状态；Mock 模式不保存/发送凭据。
- `features/chat/ChatPage.tsx`：HTTP 用户可读取和选择已开放的文本模型；发送仍禁用，S2 普通对话未实现。
- `scripts/start-dev-api.ps1` 与原 Rust 批处理：启动时可使用服务端环境密钥，或生成/复用忽略的 `.atelier/credential.key`。密钥/数据库需单独保护与恢复，不随 Git 同步。
- [S1 使用/API/测试说明](docs/S1_PROVIDER_SETUP.md) 已写入，README/总计划/项目进度同步更新。

### 本步最终验证回执

- `npm --prefix apps/web test`：16 个 TS/TSX 转译与原 Mock/HTTP 回归通过；新增供应商端点/方法/revision/403、保留密钥、Mock 禁用和不写本地存储断言通过。
- `npm --prefix apps/web run typecheck`：通过。
- `cargo test --locked --offline --manifest-path services/api/Cargo.toml`：**8 项通过、1 项数据库测试默认忽略**。
- `./scripts/test-providers.ps1 -PgBin 'E:/softs/postgresql/bin'`：独立 PostgreSQL **12.18** 真实生产路由集成 **1 项通过**；测试脚本另建回环临时实例和 atelier_test_* 库，结束后关闭。原系统 PostgreSQL 服务未启动或修改。
- 集成覆盖：无会话/普通用户/禁用用户拒绝、Origin 拒绝、无加密密钥不落库、密文、连接目录、保存/保留/移除、重开配置、revision 冲突、停用供应商/模型后的公开过滤、审计不含凭据。
- `cargo clippy --locked --offline --manifest-path services/api/Cargo.toml --all-targets -- -D warnings`：通过。新增 Rust 模块 rustfmt 已执行；数据库集成测试不计入上面的 8 项。
- 使用的供应商仅为本机合成回环服务，没有读取/调用用户真实供应商密钥。

## 仍待验收，不能标完整用户功能通过

- Tabbit 再次报 `Target.createTarget: Task-scoped CDP command could not be dispatched`，任务已正常 finish，未能进行真实页面点击/视觉验收。
- 本次 UI 试验端口 5174 已被其他进程占用，新启动进程自行退出，未停止占用该端口的已有服务。没有证明浏览器页面运行成功。
- 尚未用实际供应商完成 GET /models 测试；仅已验证回环协议和真实数据库链路。S1 代码/数据库子范围通过，完整浏览器/真实供应商验收仍待完成。
- 普通对话发送、压缩、工具、长记忆、Google、正式钱包和支付仍未接入。
- 既有画布保存、冲突恢复和 count>1 多产物问题保留在 S4。

## 下一项与通过标准

1. **先补 S1 浏览器与真实供应商验收。** 按 [接入说明](docs/S1_PROVIDER_SETUP.md) 启动开发数据库/API，Web 使用 VITE_API_MODE=http；登录管理员，保存供应商、测试连接、设置/开放文本模型，刷新确认保存；普通身份应无法管理，并可在对话页读取开放模型。密钥通过管理表单填写，不在聊天或 Git 中传递。
2. UI 验收包括失败/版本冲突、保留/移除密钥、供应商/模型启停及公开目录过滤。Tabbit 若仍不可用，明确记录环境限制，不把类型检查冒充浏览器验收。
3. **随后 S2 / R3 基础普通对话。** 从服务端内部文本模型 ID 解析已开放配置/密钥；先单轮回复，再流式、停止、有界多轮和错误，保持工具/压缩/记忆/计费关闭。入口：providers.rs、agent.rs、ChatPage.tsx、ShellPilot provider 来源。
4. S2 先建立回环供应商测试，验证模型绑定、停用、跨用户/切会话、停止与上游错误；有真实服务后单列真实对话验收。不能因 GET /models 通过就把 chat feature 标 true。
5. S3 再做 R2 持久历史/事件和 R5 压缩；S4 接画布/素材/生成与 R4 工具审批；注册、积分仍后置。

## 仓库与文件状态

- 分支 main，origin 为 `https://github.com/jasonlat-dot/atelier_canvas.git`。本步开始基线 b94f88d；本次 S1 作为后续提交同步，最新 HEAD/推送状态以 Git 核对，避免自引用哈希。
- 本次修改了前后端、迁移、依赖锁文件、脚本、示例配置和文档；`.atelier/`、真实 .env、依赖与 target 已忽略。
- 本机测试脚本的 PostgreSQL 路径只是实测参数；跨电脑用自己的 bin 路径或 PG_BIN。已提取核心代码自包含，后续迁移来源见 third_party/shellpilot/SOURCES.json；未修改 ShellPilot 来源仓库。

## 继续指令

> 继续 Atelier。先读 AGENTS.md、nextStep.md、docs/PROJECT_DEVELOPMENT.md、docs/AGENT_PROGRESS.md。S1 文本供应商/目录代码与真实 PostgreSQL 测试已过，浏览器和真实供应商仍待验收；先核对这些状态，再推进 S2 基础普通对话。保持用户确定的顺序，每完成一步确认通过后更新 nextStep.md，不打包、不生产部署，保护既有修改与凭据。

历史 R0/R1、文档与仓库初始化、顺序调整的详情保存在 [项目进度](docs/AGENT_PROGRESS.md) 和 [Agent 子计划](docs/AGENT_REUSE_PLAN.md)。