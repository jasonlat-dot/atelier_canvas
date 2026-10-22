# S1 供应商与文本模型接入

日期：2026-10-22。功能范围：本机开发身份下的管理员供应商/文本模型配置、连接测试和用户开放目录。普通对话发送留到 S2；Google、积分和支付仍后置。

## 入口与目录

- `/admin/providers`：新增/编辑/启停文本供应商，替换/保留/移除密钥，测试已保存配置。
- `/admin/models`：配置文本模型 ID/展示名称/上下文容量/输出上限/开放状态；保留原图片/视频演示模型区域。
- `/chat`：HTTP 模式读取已开放文本模型，可选择；发送仍禁用，等待 S2。
- Mock 模式不会保存或发送供应商密钥，管理页提示切到 HTTP 模式。

## 运行准备

1. 按 README 准备开发 PostgreSQL。已有数据库由启动时 SQLx migrate 追加 `202610220001_text_providers.sql`，不覆盖初始迁移。
2. 从项目根目录执行 `./scripts/start-dev-api.ps1`，或双击原 Rust 启动批处理。
3. 脚本使用已有 DATABASE_URL；未设置时用示例本机数据库地址。DEV_AUTH=true 只用于本机开发。
4. 脚本在未提供 `ATELIER_CREDENTIAL_KEY` 时生成本机 `.atelier/credential.key`，并注入当前服务进程。文件已忽略，不能提交；需要私下备份并保护。丢失/变更密钥后，已有凭据无法解密，不能假装恢复。
5. 直接 `cargo run` 也可启动，但保存 API Key 前需要通过服务端环境提供 base64 编码的 32 字节 `ATELIER_CREDENTIAL_KEY`。无密钥时拒绝存储凭据，不退回明文。
6. Web 以 VITE_API_MODE=http 运行，登录固定管理员或超级管理员开发身份；普通身份不可访问管理接口。

环境模板只是说明，不会被 Cargo 自动读取。不要把服务端加密密钥写进 VITE_ 变量。不同电脑克隆源码后，需自行恢复数据库与服务端密钥；Git 不同步这些数据。

## 配置语义

- 本阶段仅支持 `chat_completions` 兼容文本渠道。基础地址例如 `https://api.example.com/v1`，不含 `/models` 或 `/chat/completions`。
- 只读连接测试 GET 基础地址下的 `models`，验证目录结构并返回可用模型 ID；不发送生成请求，不代表某个模型已完成真实对话验收。
- HTTPS 地址支持管理员配置；HTTP 仅允许 DEV_AUTH 下的回环地址。URL 禁止用户信息、query 与 fragment；请求不跟随重定向，不使用隐式代理。自签名证书不会被绕过。
- 密钥输入留空表示保留已存值；从接口只返回 hasApiKey，不回传原文或末尾片段。更换目的地址时，必须同时替换或移除原密钥，避免把旧凭据无意发送到新地址。
- AES-256-GCM 随机 nonce，并以供应商 UUID 绑定认证数据；复制密文到其他供应商或篡改密文会解密失败。
- 供应商/模型采用 revision 乐观锁；冲突要求刷新再编辑。供应商配置变化会清除旧的连接测试状态；迟到测试结果不能覆盖新 revision。
- 模型开放目录只返回内部 ID、展示名和容量配置；模型/供应商都启用且满足凭据要求才可见。上游模型 ID、Base URL 和凭据不进入普通用户目录。
- 最小管理审计与配置同事务提交；审计记录操作、对象、revision 与启停状态，不含密钥。
- 连接错误只返回分类码（认证/限流/超时/连接/格式/重定向等），不透传上游可能反射凭据的响应正文。

## API 契约

统一前缀 `/api/v1`，身份来自当前 Cookie Session。管理接口在服务端读取当前用户状态和角色；写操作对存在的 Origin 进行匹配，默认开发 Web Origin 为 `http://127.0.0.1:5173`，可用服务端 WEB_ORIGIN 配置。

| Method | 路径 | 请求/返回 |
|---|---|---|
| GET | `/admin/providers` | 管理员列表，hasApiKey、revision 与最近测试状态 |
| POST | `/admin/providers` | 新建，revision=0 |
| PUT | `/admin/providers/{id}` | 编辑，携带当前 revision |
| POST | `/admin/providers/{id}/test` | `{revision}`；返回 ok/code/models/latencyMs |
| GET | `/admin/text-models` | 管理员文本模型列表 |
| POST | `/admin/text-models` | 新建，revision=0 |
| PUT | `/admin/text-models/{id}` | 编辑，携带当前 revision |
| GET | `/text-model-catalog` | 已登录普通用户可见的开放文本模型 |

供应商写入字段：name、protocol、baseUrl、enabled、requireApiKey、revision；apiKey 可选，clearApiKey 默认 false，两者不能同时提交。要求 Key 却没有有效保存值时拒绝。

文本模型写入字段：providerId、modelId、label、enabled、contextWindow、maxOutputTokens、revision。容量 1024–4000000，输出为正数且小于容量。providerId/modelId 组合唯一。未知字段、无效文本和失效 revision 不会被静默接受。

接口与原 `/model-catalog` 图片/视频演示目录独立，不把文本模型混入现有 MediaKind 或生成报价。

## 验证方式

```powershell
# 无数据库的正常单元/回环测试、前端回归与类型检查
./scripts/verify-agent.ps1
# 真实 PostgreSQL 集成（在系统 PostgreSQL bin 位于其他目录时替换参数）
./scripts/test-providers.ps1 -PgBin 'path/to/postgresql/bin'
```

测试脚本使用 initdb 新建临时回环实例和独立 `atelier_test_*` 数据库，不启动已有系统服务。创建的实例在 finally 中关闭，诊断文件留在系统临时目录。也可显式指定独立测试库 ATELIER_TEST_DATABASE_URL；测试拒绝其他库名。集成用例按独立数据库设计，每次需干净测试库。

本机已定位到原安装 PostgreSQL 12.18（原服务为 Stopped）；首轮完整生产路由集成已通过。最终结果、通过数量及剩余项见 nextStep 与项目进度；不能把数据库测试当作浏览器交互或真实供应商验证。

## 当前验收边界

已实现代码包括管理表单、配置接口、加密、目录过滤和后端连接探测；尚无真实供应商凭据/实际对话验收。浏览器 Tabbit 仍有 Target.createTarget 派发失败，页面点击与视觉需继续验证。复杂供应商池、自动调度/熔断、更多协议及 S2 普通对话不在本次首个渠道范围内。
## 最终本机回执（2026-10-22）

- 前端转译/HTTP/Mock 回归与 TypeScript 通过；API 常规测试 8 项通过，数据库用例默认 ignored，另行在新建 PostgreSQL 12.18 实例执行 1 项通过。
- 严格 Clippy all-targets/-D warnings 通过。真实 10 秒超时、连接失败、Origin 与缺少加密配置等边界已有测试。
- UI 试验发现 5174 已占用，新增进程退出，未影响已有服务；Tabbit 同时未能建页。当前没有浏览器点击或真实供应商成功回执，状态保持待验收。
