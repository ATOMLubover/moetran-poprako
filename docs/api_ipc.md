# API 汇总

## IPC API 列表

|          Vue 前端函数名           |                   Rust 后端函数名                   | 完成情况（Y/N） |
| :-------------------------------: | :-------------------------------------------------: | :-------------: |
|             syncUser              |                      sync_user                      |        Y        |
|            getUserInfo            |                    get_user_info                    |        Y        |
|           getUserTeams            |                   get_user_teams                    |        Y        |
|        getTeamProjectSets         |                get_team_project_sets                |        Y        |
|          getTeamProjects          |                  get_team_projects                  |        Y        |
|          getUserProjects          |                  get_user_projects                  |        Y        |
|      getUserProjectsEnriched      |             get_user_projects_enriched              |        Y        |
|            getCaptcha             |                     get_captcha                     |        Y        |
|        aquireMoetranToken         |                    aquire_token                     |        Y        |
|          getMoetranToken          |                  get_moetran_token                  |        Y        |
|         saveMoetranToken          |                 save_moetran_token                  |        Y        |
|        removeMoetranToken         |                remove_moetran_token                 |        Y        |
|          getPoprakoToken          |                  get_poprako_token                  |        Y        |
|         savePoprakoToken          |                  get_poprako_token                  |        Y        |
|        removePoprakoToken         |                  get_poprako_token                  |        Y        |
|         loadUser (helper)         |  PanelView 逻辑已接入 getUserInfo，仅缺登录态守卫   | Y (前端 helper) |
|        loadTeams (helper)         | PanelView 逻辑已接入 getUserTeams，仍有 Mock 组补全 |    部分完成     |
|     loadUserProjects (helper)     |    PanelView 逻辑已接入 getUserProjectsEnriched     | Y (前端 helper) |
|     __mockSearchMembersByName     |    MemberSelector / ProjectFilterBoard 成员搜索（已改用 searchMembersByName IPC） |   Y (已替换)   |
| handleCreateProject (mock submit) |          ProjectCreatorView 创建项目/提交           |        N        |
|     __mockFetchProjectSummary     |             ProjectDetailView 项目概要              |        N        |
|     __mockFetchMarkerDetails      |           ProjectDetailView 标记详情/图表           |        N        |
|      __mockFetchProjectPage       |            TranslatorView 逐页加载/提交             |        N        |

## IPC 设计思路

由于客户端后端会请求两种服务器，一个是 Moetran 服务器，一个是 PopRaKo 服务器，因此在 IPC 请求返回前，需要对有需要的组合结果接口返回值进行拼装。

Moetran 服务器是主服务器，它有全量的数据，而 PopRaKo 服务器只有部分必须要的补充数据，因此某些汉化组、项目集、项目在 PopRaKo 接口中可能是没有对应的数据的，此时需要进行标记。

比如获取用户当前参加项目的 IPC 接口，会先访问 Moetran 的接口拿到项目的列表，随后收集所有项目的 ID 形成数组传给 PopRaKo 接口，以获取到这些项目的补充信息（可能有的项目没有补充信息，则在 IPC 接口中每个项目保留一个字段，用来标识其是否有 PopRaKo 补充信息），组装后返回给 Vue 前端。

## 根据服务器接口适应 IPC 接口和前端

之前由于大量使用 mock，一些细枝末节的功能并没有实现在前端中。在根据 api_poprako.md 修改 IPC 后，前端的一些功能也需要相应改变。
