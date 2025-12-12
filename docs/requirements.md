# 需求文档

## 展示用项目列表

你需要创建一个叫做 src/views/ShownProjectListView.vue 的文件，存放这个项目列表的 vue 文件。

它与 src/components/ProjectList.vue 的核心区别在于：

- 不支持搜索。
- 显示信息更加详细。

ShownProject 仅支持 PopRaKo 登记的项目，因此无须考虑对 Moetran 项目的兼容性。

当然，不是说完全不获取尨译对应的数据，因为 Moetran 承载了翻校和图片 CDN 的数据。

## 核心展示 DTO 的接口

以下字段仅作演示，与最终源码中的可以不同：

```ts
export type WorkPhase = 'translate' | 'proof' | 'typeset' | 'review' | 'publish';
export type PhaseStatus = 'unset' | 'pending' | 'wip' | 'completed';

export interface PhaseChip {
  phase: WorkPhase;
  status: PhaseStatus;
  member_names: string[];
}

export interface ShownProjectInfo {
    id: string;
    title: string;
    phases: PhaseChip[];

    translated_source_count: number;
    proofread_source_count: number;
}
```

它的核心效果是，既展示项目名称、项目的职位状态，也展示各职位的成员名称。另外还会显示来自尨译获取到的翻校状态。

## 布局与样式相关

表格的整体布局是若干行、两列的 grid。

其中每个 entry div 的布局为：左 div 第一行标题，第二行 phase chips，然后是翻译源数量和校对源数量的 chip。右 div 是详情按钮。

关于如何显示 chips 和如何调用 ProjectDetailView，参考 src/views/PanelView.vue 的实现。

## 入口

在 PanelView 的 PopRaKo 筛选控制板（src/components/ProjectFilterBoard.vue）第一行标题 “PopRaKo 筛选控制板” 同一行的右对齐位置添加一个按钮 “纵览表格”。点击后，整个  <aside class="right-column"> 和 <main class="projects-main"> 区域都隐藏，留下的区域用来放置 ShownProjectList。

仅支持汉化组状态下使用（teamId 不为 undefined）。

## 出口

在 ShownProjectList 的右上角添加 “返回” 按钮，点击后返回原本正常的 PanelView（默认的项目列表 + PopRaKo 筛选控制板状态）。

## 后端 IPC 相关

这部分可以参考 ProjectList 的获取方式，但是要使用一个新的 PopRaKo 服务器接口，如下：

### 8. 列出团队项目

按团队分页列出项目，结果默认按 `f_created_at` 倒序返回，最近创建的项目在前。

| 方法 | 路径               | 认证 | 查询参数                                   |
| ---- | ------------------ | ---- | ------------------------------------------ |
| GET  | `/api/v1/projs`    | 需要 | `team_id=<string>`, `page=<int>`, `limit=<int>` |

查询参数：

- `team_id`（必填）：所属团队 ID
- `page`（可选）：页码，从 1 开始，默认 1
- `limit`（可选）：每页条数，默认 10

成功响应：

```json
{
    "code": 200,
    "data": [
        {
            "proj_id": "proj_id_1",
            "proj_name": "【3-5】章节1对话",
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
                    "user_id": "user_123",
                    "username": "alice",
                    "is_admin": false,
                    "is_translator": true,
                    "is_proofreader": false,
                    "is_typesetter": false,
                    "is_redrawer": false,
                    "is_principal": true
                }
            ]
        }
    ]
}
```

错误响应：

| 场景         | code |
| ------------ | ---- |
| 未认证       | 401  |
| team_id 缺失 | 400  |
| 内部错误     | 500  |

你可能需要在 Rust 后端封装一个新的 tauri::command。别忘了注册到 lib.rs 中。
