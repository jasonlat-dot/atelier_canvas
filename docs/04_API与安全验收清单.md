# 04 · Rust MVP API 契约与安全验收清单

## 1. 已实现的开发用服务端端点

前缀 `/api/v1`，默认只监听 `127.0.0.1:8080`。

| Method | Route | 当前能力 | 说明 |
|---|---|---|---|
| GET | `/health` | ✅ | 开发健康检查 |
| POST | `/dev/login` | ✅ DEV ONLY | 固定三种演示身份；`DEV_AUTH=true` + loopback 才允许 |
| GET | `/me` | ✅ | Cookie → 服务端 Session → 用户 DB / 禁用状态 |
| GET | `/projects` | ✅ | 所属项目列表（owner_id 过滤） |
| POST | `/projects` | ✅ | 创建归属当前 Session 用户的画布 |
| GET | `/projects/{id}/graph` | ✅ | 读取画布，仅允许所有者 |
| PUT | `/projects/{id}/graph` | ✅ | 带 `If-Match` 的版本条件更新（冲突 409） |
| POST | `/generation/quotes` | ✅ 开发价 | 后端计算价格，而不是信任客户端传来的成本 |
| POST | `/generation/jobs` | ✅ Mock Worker | 身份、项目归属、模型类型、积分事务、幂等键 |
| GET | `/generation/jobs` | ✅ | 当前用户的生成任务列表 |
| POST | `/generation/jobs/{id}/cancel` | ✅ | 事务内退款 + 终态幂等 |
| GET | `/wallet` | ✅ | 余额和流水，用户作用域 |
| GET | `/assets` | ✅ | 生成 Mock 资产目录 |
| GET | `/model-catalog` | ✅ | 当前硬编码的演示模型目录 |

后端数据采用 PostgreSQL。所有 `/api/v1` 真实资源路由从服务端 Session 解析用户身份，**不会从请求 JSON 里信任 ownerId 或 role**。

## 2. 会话登录流程（仅本地）

`POST /api/v1/dev/login`：

```json
{"userId":"user-demo"}
```

返回 `Set-Cookie: atelier_sid=...; HttpOnly; SameSite=Strict`。之后浏览器带 Cookie 访问 `/me`，本地 Session 表寻找身份，再到 PostgreSQL 读取 `role/status/credits`。不存在 Cookie，返回 401；账号禁用则 403。

生产替换为 Google Authorization Code + PKCE。必须逐项校验：`iss`、`aud`、签名、`exp`、`nonce`、`state`、`sub`、`email_verified`。只允许 Gmail 邮箱时以 Google 已验证的邮箱判断 `@gmail.com`，并以 `sub` 唯一标识，不要只凭邮箱文本。支持关闭注册、Session 撤销、CSRF、管理员 MFA、速率限制。

开发 Session 保存在进程内存，重启后自动失效。**当前仅有本机调试用途，未达到生产安全标准。**

## 3. 典型接口交互

```bash
# 1. 开发登录：使用 Cookie jar
curl -c cookies.txt -H 'Content-Type: application/json' \
  -d '{"userId":"user-demo"}' http://127.0.0.1:8080/api/v1/dev/login

# 2. 读取当前用户
curl -b cookies.txt http://127.0.0.1:8080/api/v1/me

# 3. 创建画布
curl -b cookies.txt -H 'Content-Type: application/json' \
  -d '{"name":"旅行广告"}' http://127.0.0.1:8080/api/v1/projects

# 4. 从上一步获取 project id，创建示例任务
curl -b cookies.txt -X POST -H 'Content-Type: application/json' \
 -H 'Idempotency-Key: test-project-job-001' \
 -d '{"projectId":"<UUID>","nodeId":"node-1","kind":"image","prompt":"cinematic mountain","quality":"medium","ratio":"16:9","model":"image-demo","duration":6,"count":1}' \
 http://127.0.0.1:8080/api/v1/generation/jobs

# 5. 查询任务与余额
curl -b cookies.txt http://127.0.0.1:8080/api/v1/generation/jobs
curl -b cookies.txt http://127.0.0.1:8080/api/v1/wallet
```

## 4. 保存修订版本

- `GET /projects/{id}/graph` 得到 `revision: 7`。
- `PUT /projects/{id}/graph` HTTP header `If-Match: 7`，请求 `{ "graph": {...}, "revision": 7 }`。
- 版本匹配时返回新修订 `8`；其他人尝试访问或旧版提交会返回非成功状态，不能覆盖现有数据。
- 后续要将 403 / 404（隐藏资源存在性）和 409 区分得更精确。

## 5. QA 计划（上线前必须执行）

### 鉴权 / 角色

- [ ] 没有 Cookie / 假 Cookie / 过期 Cookie → 401。
- [ ] 直接手写浏览器请求修改 `role` / `email` 不会改变权限。
- [ ] 普通用户访问管理路由 → 服务端返回 403（管理 API 实现后）。
- [ ] 禁用用户再次提交任务 / 获取私有素材 → 拒绝。
- [ ] 关闭注册时新 Gmail 用户无法自动进入系统，旧用户仍按配置允许登录。
- [ ] CSRF 与跨域 Origin 测试，所有修改操作校验 Origin + CSRF。
- [ ] 账号切换及角色提升不能靠前端变量控制。

### 画布 / 素材

- [ ] 两个用户使用相同 projectId 访问只能看到自己授权的项目。
- [ ] 旧版 revision 保存 → 409，不会覆盖最新画布。
- [ ] 节点 / 连线数量和数据体积超限 → 400。
- [ ] 图片/视频对象与项目引用都要校验 `owner_id`。
- [ ] 直传后检查文件实际类型 / 病毒 / 大小 / 视频时长。
- [ ] 刷新 / 多标签页 / 浏览器关闭后项目与任务能够恢复。

### 钱包和任务

- [x] 前端 Mock：同一个幂等键不重复扣费（本环境运行通过）。
- [x] 前端 Mock：取消不重复退款（本环境运行通过）。
- [x] 前端 Mock：版本冲突、不同用户画布隔离（本环境运行通过）。
- [ ] PostgreSQL：两个并发请求同一个幂等键，只有一笔扣费。
- [ ] PostgreSQL：积分不足时事务整体回滚，没有孤立任务。
- [ ] PostgreSQL：cancel 与 complete 竞争，不会扣退错账。
- [ ] PostgreSQL：历史账本不能删除 / 更新（生产账本重构后）。
- [ ] 服务端重算报价，篡改客户端 `cost` 无效。
- [ ] 任务跨用户查询 / 取消 → 拒绝。
- [ ] SSE 断线重连有事件游标，任务状态不会乱序。
- [ ] 真实支付回调重复投递不重复充值。

### 模型 / 视频

- [ ] 不允许任意 URL 作为模型 Base URL，防 SSRF。
- [ ] 未公开模型不能通过构造 ID 绕开。
- [ ] 参考图只能使用有权限的 assetId，不允许任意对象存储路径。
- [ ] 上传超长或损坏视频不会阻塞主 HTTP 线程。
- [ ] 超时、失败、用户取消对应唯一一次退款 / 结算。
- [ ] 服务端执行模型供应商分流，并记录调度原因 / 配额。

## 6. 下一轮优先任务

1. 首先在真实 Windows 开发机上跑通 `npm run typecheck/build`、`cargo check/test`；处理实测版本兼容。
2. 把 Rust `main.rs` 拆为 auth、canvas、wallet、generation、storage 分层，加集成测试。
3. 做 Google OIDC + 用户注册开关 + 管理端实际权限和安全审计。
4. 添加 S3 上传与文件校验，再接真实图片和视频生成。

## 7. 已知非生产限制

当前 SQL 只有钱包余额与持有扣减、取消返还，不具备完整的冻结分录 + 结算分录 + 供应商回执去重架构；Mock Worker 只推演进度。`POST /dev/login` 允许从固定三种演示身份登录，只能在本机使用。没有真实 Google 登录、会员订阅、支付、API Key 加密、Redis、容灾或正式计费。**严禁把这个阶段直接公开给用户。**
