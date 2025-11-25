# PopRaKo API 文档

所有返回统一包装：

```json
// 成功示例
{
    "code": 200,
    "data": { /* 与具体接口相关的结构 */ }
}

// 失败示例
{
    "code": 401,
    "message": "Invalid password"
}
```

HTTP 状态码与 `code` 字段保持一致（便于 API 网关或客户端根据任一来源判断）。

认证：除同步接口外，其余接口都需要在 Header 中携带：

```text
Authorization: Bearer <JWT>
```

JWT `sub` 字段即用户 `user_id`。

---

## User 部分

### 1. 同步用户信息

用于将外部（尨译）用户数据同步到本服务数据库，如果用户已存在则校验密码并返回新 token；如果不存在则创建后返回 token。

| 方法 | 路径                | 认证   | 幂等性                                          |
| ---- | ------------------- | ------ | ----------------------------------------------- |
| POST | `/api/v1/user/sync` | 不需要 | 幂等（同一 user_id + 正确密码重复请求返回 200） |

请求体（JSON）：

```json
{
    "user_id": "user_123",      // 唯一用户标识
    "username": "alice",        // 用户名（可与 user_id 相同）
    "email": "alice@example.com",
    "password": "plaintext"     // 明文密码（传输时需 HTTPS）
}
```

成功响应：

```json
// 已存在用户且密码正确 => 200
{
    "code": 200,
    "data": { "token": "<jwt-token>" }
}

// 新建用户 => 201
{
    "code": 201,
    "data": { "token": "<jwt-token>" }
}
```

错误响应：

| 场景           | code | message               |
| -------------- | ---- | --------------------- |
| 密码错误       | 401  | Invalid password      |
| JSON 解析错误  | 422  | Unprocessable entity  |
| 服务端内部错误 | 500  | Internal server error |

示例 cURL：

```bash
curl -X POST https://api.poprako.example/api/v1/user/sync \
    -H "Content-Type: application/json" \
    -d '{"user_id":"user_123","username":"alice","email":"alice@example.com","password":"secret"}'
```

### 2. 获取用户信息

返回用户的基本资料以及其所属的汉化组列表。

| 方法 | 路径                | 认证 |
| ---- | ------------------- | ---- |
| GET  | `/api/v1/user/info` | 需要 |

成功响应：

```json
{
    "code": 200,
    "data": {
        "user_id": "user_123",
        "username": "alice",
        "email": "alice@example.com",
        "teams": [
            { "team_id": "team_a", "team_name": "A组" },
            { "team_id": "team_b", "team_name": "B组" }
        ]
    }
}
```

可能的错误：

| 场景                               | code |
| ---------------------------------- | ---- |
| 未认证/Token非法                   | 401  |
| 用户不存在（理论上同步后才会调用） | 404  |
| 内部错误                           | 500  |

示例 cURL：

```bash
curl -H "Authorization: Bearer <jwt>" \
    https://api.poprako.example/api/v1/user/info
```

---

## Member 部分

### 3. 获取自己在特定汉化组中的成员信息

根据当前登录用户与指定 `team_id`，返回该用户在该汉化组的角色标记。若该团队不存在或用户不在团队中，则可能返回 404。

| 方法 | 路径                  | 认证 | 查询参数           |
| ---- | --------------------- | ---- | ------------------ |
| GET  | `/api/v1/member/info` | 需要 | `team_id=<string>` |

cURL 调用示例：

```bash
curl -G https://api.poprako.example/api/v1/member/info \
    -H "Authorization: Bearer <jwt>" \
    --data-urlencode "team_id=team_a"
```

成功响应：

```json
{
    "code": 200,
    "data": {
        "member_id": "member_789",
        "is_admin": false,
        "is_translator": true,
        "is_proofreader": false,
        "is_typesetter": false,
        "is_principal": false
    }
}
```

错误响应：

| 场景                       | code | 说明                   |
| -------------------------- | ---- | ---------------------- |
| 未认证                     | 401  | 缺失或非法 JWT         |
| team_id 缺失或格式错误     | 422  | 参数无法解析           |
| 团队不存在或用户不在该团队 | 404  | 未找到对应成员记录     |
| 内部错误                   | 500  | 一般为数据库或服务异常 |

---

### 4. 按条件筛选成员列表

按团队、职位及用户名模糊查询成员列表，返回简要信息（`member_id` 与 `username`）。

| 方法 | 路径                     | 认证 | 查询参数                                                                 |
| ---- | ------------------------ | ---- | ------------------------------------------------------------------------ |
| GET  | `/api/v1/members`  | 需要 | `team_id`（必填）、`position`（可选）、`fuzzy_name`（可选）、`page`、`limit` |

查询参数说明：

- `team_id`：所属团队 ID，必填。
- `position`：可选职位过滤，取值：`translator` / `proofreader` / `typesetter` / `principal`。
    - 若不传，则不按职位过滤。
- `fuzzy_name`：可选用户名模糊匹配关键字，对应 `u.f_username ILIKE '%fuzzy_name%'`。
    - 若与 `position` 一起传入，则先按职位，再按用户名模糊过滤。
    - 若仅传 `fuzzy_name`，则仅按用户名模糊 + team 过滤。
- `page`：页码，从 1 开始，默认 1。
- `limit`：每页条数，默认 10。

成功响应示例：

```json
{
    "code": 200,
    "data": [
        { "member_id": "member_1", "username": "alice" },
        { "member_id": "member_2", "username": "alice_2" }
    ]
}
```

错误响应：

| 场景                | code | 说明                      |
| ------------------- | ---- | ------------------------- |
| 未认证              | 401  | 缺失或非法 JWT            |
| team_id 缺失        | 400  | Missing team_id parameter.|
| position 非法取值   | 400  | Invalid position          |
| 内部错误            | 500  | 数据库或服务异常          |

示例 cURL：

```bash
curl -G "https://api.poprako.example/api/v1/members" \
    -H "Authorization: Bearer <jwt>" \
    --data-urlencode "team_id=team_a" \
    --data-urlencode "position=translator" \
    --data-urlencode "fuzzy_name=ali" \
    --data-urlencode "page=1" \
    --data-urlencode "limit=20"
```

---

## ProjSet 部分

### 4. 创建项目集 (Project Set)

在指定团队下创建一个新的项目集。仅团队管理员可创建。首先在尨译上创建对应的 project-set，再在本服务中记录 serial 并初始化后续项目索引。

| 方法 | 路径                     | 认证 | 幂等性                               |
| ---- | ------------------------ | ---- | ------------------------------------ |
| POST | `/api/v1/projset/create` | 需要 | 非幂等（重复名称取决于外部服务策略） |

请求体（JSON）：

```json
{
    "projset_name": "主线文本",
    "projset_description": "主线剧情相关文本",
    "team_id": "team_a",
    "mtr_token": "<moetran-access-token>" // 用于调用尨译外部接口
}
```

成功响应（创建）：

```json
{
    "code": 201,
    "data": { "projset_serial": 3 }
}
```

错误响应：

| 场景                        | code | message                                        |
| --------------------------- | ---- | ---------------------------------------------- |
| 未认证 / Token 非法         | 401  | (无或标准未认证语义)                           |
| 非管理员用户尝试创建        | 403  | Only team admins can create project sets.      |
| JSON 解析失败               | 422  | Unprocessable entity                           |
| 外部尨译接口失败 / 内部错误 | 500  | Internal server error (不直接暴露外部错误细节) |

cURL 示例：

```bash
curl -X POST https://api.poprako.example/api/v1/projset/create \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{"projset_name":"主线文本","projset_description":"主线剧情相关文本","team_id":"team_a","mtr_token":"<moetran-token>"}'
```

---

## Proj 部分

### 5. 创建项目 (Proj)

在已有项目集中创建一个具体项目。仅团队管理员可创建。会先调用尨译创建实际项目，再记录到本系统并增加对应项目集的 index 计数。

| 方法 | 路径                  | 认证 | 幂等性 |
| ---- | --------------------- | ---- | ------ |
| POST | `/api/v1/proj/create` | 需要 | 非幂等 |

请求体（JSON）：

```json
{
    "proj_name": "章节1对话",
    "proj_description": "第一章主要对话文本",
    "team_id": "team_a",
    "projset_id": "projset_123",        // 已存在的项目集 ID（来自尨译）
    "mtr_auth": "<moetran-access-token>",
    "workset_index": 0,                  // 预留字段（当前未在示例中使用业务含义）
    "source_language": "ja",
    "target_languages": ["zh_CN"],
    "allow_apply_type": 1,               // 具体枚举语义参考后续扩展（moetran module）
    "application_check_type": 0,         // 申请审核方式枚举
    "default_role": "translator"        // 新申请默认角色
}
```

成功响应：

```json
{
    "code": 201,
    "data": {
        "proj_serial": 12,
        "projset_index": 5
    }
}
```

错误响应：

| 场景                          | code | message                                                |
| ----------------------------- | ---- | ------------------------------------------------------ |
| 未认证 / Token 非法           | 401  | (无或标准未认证语义)                                   |
| 非管理员用户尝试创建          | 403  | Only team admins can create project sets. (待修正文案) |
| 关联项目集不存在 / 不匹配团队 | 404  | Resource not found                                     |
| JSON 解析失败                 | 422  | Unprocessable entity                                   |
| 外部尨译接口失败 / 内部错误   | 500  | Internal server error                                  |

说明：当前后端返回的 403 message 仍为 *Only team admins can create project sets.*，属于实现时的文案复用，可在后续迭代中改为 *Only team admins can create projects.*。

cURL 示例：

```bash
curl -X POST https://api.poprako.example/api/v1/projs \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
        -d '{"proj_name":"章节1对话","proj_description":"第一章主要对话文本","team_id":"team_a","projset_id":"projset_123","mtr_auth":"<moetran-token>","source_language":"ja","target_languages":["zh_CN"],"allow_apply_type":1,"application_check_type":0,"default_role":"translator"}'
```

### 6. 按 ID 批量获取项目详情

根据一组项目 ID，返回每个项目的基础信息、各流程状态以及参与成员列表。内部会联表查询 `t_proj`、`t_proj_assgin`、`t_member`、`t_user` 等表。

| 方法 | 路径                  | 认证 |
| ---- | --------------------- | ---- |
| POST | `/api/v1/projs/batch` | 需要 |

请求体（JSON）：

```json
{
    "ids": ["proj_id_1", "proj_id_2"]
}
```

成功响应示例：

```json
{
    "code": 200,
    "data": [
        {
            "proj_id": "proj_id_1",
            "proj_name": "章节1对话",
            "description": null,
            "projset_id": "projset_123",
            "projset_serial": 3,
            "projset_index": 5,
            "translating_status": 0,
            "proofreading_status": 0,
            "typesetting_status": 0,
            "reviewing_status": 0,
            "is_published": false,
            "members": [
                {
                    "member_id": "member_1",
                    "username": "alice",
                    "is_admin": false,
                    "is_translator": true,
                    "is_proofreader": false,
                    "is_typesetter": false,
                    "is_principal": true
                }
            ]
        }
    ]
}
```

错误响应：

| 场景          | code | 说明                     |
| ------------- | ---- | ------------------------ |
| 未认证        | 401  | 缺失或非法 JWT           |
| JSON 解析失败 | 422  | 请求体格式错误或缺失 ids |
| 内部错误      | 500  | 数据库或服务异常         |

### 7. 更新项目流程状态

仅项目负责人（principal）可以修改项目的某个流程状态（翻译/校对/排版/Review）。状态值采用枚举：`0=未开始`、`1=进行中`、`2=已完成`。

| 方法 | 路径                             | 认证 |
| ---- | -------------------------------- | ---- |
| PUT  | `/api/v1/projs/{proj_id}/status` | 需要 |

请求体（JSON）：

```json
{
    "proj_id": "proj_id_1",           // 会被路径中的 proj_id 覆盖
    "status_type": "translating",     // translating / proofreading / typesetting / reviewing
    "new_status": 1                     // 0,1,2 分别代表 未开始/进行中/已完成
}
```

成功响应：

```json
{
    "code": 204,
    "data": null
}
```

错误响应：

| 场景             | code | message                                            |
| ---------------- | ---- | -------------------------------------------------- |
| 未认证           | 401  | 缺失或非法 JWT                                     |
| 非项目负责人     | 403  | Only project principals can update project status. |
| status_type 非法 | 400  | Invalid status_type                                |
| JSON 解析失败    | 422  | Unprocessable entity                               |
| 内部错误         | 500  | Internal server error                              |

### 8. 标记项目为已发布

仅项目负责人可以将项目标记为已发布，操作会将 `f_is_published` 字段置为 `true`。

| 方法 | 路径                              | 认证 |
| ---- | --------------------------------- | ---- |
| PUT  | `/api/v1/projs/{proj_id}/publish` | 需要 |

请求体：无（仅依赖路径和 JWT）。

成功响应：

```json
{
    "code": 204,
    "data": null
}
```

错误响应：

| 场景         | code | message                                       |
| ------------ | ---- | --------------------------------------------- |
| 未认证       | 401  | 缺失或非法 JWT                                |
| 非项目负责人 | 403  | Only project principals can publish projects. |
| 项目不存在   | 404  | Resource not found（实现层视情况返回）        |
| 内部错误     | 500  | Internal server error                         |

---

## Assign 部分

### 9. 指派成员到项目

仅项目负责人可以为项目指派或更新成员角色。若该成员在目标团队中不存在、或不具备请求中的角色能力，则会返回相应错误。

| 方法 | 路径                             | 认证 |
| ---- | -------------------------------- | ---- |
| POST | `/api/v1/projs/{proj_id}/assign` | 需要 |

请求体（JSON）：

```json
{
    "proj_id": "proj_id_1",        // 会被路径中的 proj_id 覆盖
    "member_id": "member_1",       // 目标成员 ID
    "is_translator": true,
    "is_proofreader": false,
    "is_typesetter": false,
    "is_principal": true
}
```

成功响应：

```json
{
    "code": 204,
    "data": null
}
```

错误响应：

| 场景                               | code    | message                                     |
| ---------------------------------- | ------- | ------------------------------------------- |
| 未认证                             | 401     | 缺失或非法 JWT                              |
| 操作人不是该项目的负责人           | 403     | Only project principals can assign members. |
| 成员不属于该项目所在团队           | 400/404 | Member not found in project team            |
| 请求设置角色但成员并不具备对应能力 | 400     | 例如：Member is not a translator            |
| JSON 解析失败                      | 422     | Unprocessable entity                        |
| 内部错误                           | 500     | Internal server error                       |

---

## 公共错误语义

| code | 含义                               |
| ---- | ---------------------------------- |
| 200  | 成功                               |
| 201  | 已创建                             |
| 400  | 请求格式错误                       |
| 401  | 未认证或凭证无效                   |
| 404  | 资源不存在                         |
| 422  | 语义/字段解析失败（JSON 或 Query） |
| 500  | 服务器内部错误                     |

统一约定：

- 所有失败响应都包含 `message` 字段；
- 成功响应无 `message`（或忽略）；
- 列表等复杂接口今后会在 `data` 中扩展分页字段（如 `items`, `total`, `page`, `size`）。

---

## 枚举与取值说明

以下枚举会在多个接口的请求或响应中出现，客户端应按本节约定的取值来构造或解析字段。

### ProjStatus（项目流程状态）

用于 `ProjInfoReply` 中的各流程状态字段，以及 `MarkProjStatusPayload.new_status`：

- `0`：`NotStarted`（未开始）
- `1`：`InProgress`（进行中）
- `2`：`Completed`（已完成）

### MtrAllowApplyType（允许申请方式）

用于创建 Proj 时的 `allow_apply_type` 字段（`ProjCreatePayload` / `MtrProjectCreatePayload`）：

- `0`：`NoApply` — 不接受任何申请
- `1`：`AnyApply` — 任何人都可以申请
- `2`：`MemberOnly` — 仅组内成员可以申请

### MtrAppliCheckType（申请审核方式）

用于创建 Proj 时的 `application_check_type` 字段：

- `0`：`NonCheck` — 申请自动通过，无需审核
- `1`：`AdminCheck` — 需要管理员/负责人审核

### MtrRole（默认角色 & 角色常量）

`default_role` 字段以及内部与尨译角色对接时使用的角色常量，目前定义为一组固定字符串 ID：

- `"63d87c24b8bebd75ff934264"`：`ADMIN`        — 管理员
- `"63d87c24b8bebd75ff934265"`：`PRINCIPAL`    — 负责人（principal）
- `"63d87c24b8bebd75ff934266"`：`PROOFREADER`  — 校对
- `"63d87c24b8bebd75ff934267"`：`TRANSLATOR`   — 翻译
- `"63d87c24b8bebd75ff934268"`：`TYPESETTER`   — 嵌字 / 排版
- `"63d87c24b8bebd75ff934269"`：`INTERN`       — 实习 / 预备成员

客户端在请求体中填写 `default_role` 时必须使用上述字符串之一，否则会在反序列化阶段被直接判为非法请求。

### 语言代码（mtr_lang）

与尨译对接时推荐使用的语言代码常量（`source_language` / `target_languages`）：

- `"ja"`：日语
- `"zh-CN"`：简体中文
- `"zh-TW"`：繁体中文
- `"ko"`：韩语
- `"en"`：英语

其他语言可按尨译平台支持的语言代码扩展，客户端应与具体部署环境约定。

---

## 后续扩展计划（占位）

| 模块  | 可能新增接口                     |
| ----- | -------------------------------- |
| Team  | 创建团队、列出团队、团队成员管理 |
| Auth  | Token 刷新、注销（黑名单机制）   |
| Audit | 操作日志查询                     |

以上接口补充后应继续在本文件中以同样格式维护。
