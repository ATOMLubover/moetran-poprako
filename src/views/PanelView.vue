<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import type { ProjectSearchFilters } from '../ipc/project';
import { useToastStore } from '../stores/toast';
import { getUserInfo } from '../ipc/user';
import { getUserTeams } from '../ipc/team';
import type { ResUser } from '../api/model/user';
import type { ResTeam } from '../api/model/team';
import ProjectList from '../components/ProjectList.vue';
// 使用共享的基本项目信息类型仅在本地过滤场景；当前不直接使用
import ProjectFilterBoard from '../components/ProjectFilterBoard.vue';
import ProjectDetailView from '../views/ProjectDetailView.vue';
import ProjectCreatorView from '../views/ProjectCreatorView.vue';

// 用户信息
const user = ref<ResUser | null>(null);

// 汉化组列表
const teams = ref<ResTeam[]>([]);
// 当前选中的团队（用于成员筛选等场景；null 代表“仅看我的项目”）
const activeTeamId = ref<string | null>(null);

// 主体“当前展示”的项目列表目前由 ProjectList 自行通过 IPC 拉取。
// 这里预留 rawProjects / filteredProjects 以便后续若改回父组件统筹过滤时使用。
// 现阶段先注释掉，避免未使用变量的类型检查错误。
// const rawProjects = ref<ProjectBasicInfo[]>([]);
// const filteredProjects = ref<ProjectBasicInfo[]>([]);

// 右侧详情栏相关状态
const selectedProjectId = ref<string | null>(null);
const detailOpen = ref(false);
// 展开后右侧详情栏占整个 PanelView 宽度的比例，可调整
const DETAIL_SIDEBAR_MAX_RATIO = 0.5;
// 详情栏展开动画完成后再挂载详情视图，避免动画过程中布局抖动
const detailReady = ref(false);
const detailSidebarRef = ref<HTMLElement | null>(null);
// 右侧侧栏模式：展示项目详情或创建新项目
const detailMode = ref<'detail' | 'create'>('detail');

// 加载状态
const loadingUser = ref<boolean>(false);
const loadingTeams = ref<boolean>(false);
const loadingProjects = ref<boolean>(false);

// 依赖的 store
const toastStore = useToastStore();

// Avatar error handlers: 当图片加载失败时，将对应实体的 `has_avatar` 置为 false，以触发回退 UI（首字母）
function onUserAvatarError(): void {
  if (user.value) user.value.has_avatar = false;
}

function onTeamAvatarError(teamId: string): void {
  const t = teams.value.find(x => x.id === teamId);
  if (t) t.has_avatar = false;
}

// 载入用户信息
async function loadUser(): Promise<void> {
  loadingUser.value = true;

  try {
    user.value = await getUserInfo();
    console.log('用户信息加载成功:', user.value);
  } catch (err) {
    console.error('获取用户信息失败:', err);
    toastStore.show('获取用户信息失败', 'error');
  } finally {
    loadingUser.value = false;
  }
}

// 载入汉化组列表
async function loadTeams(): Promise<void> {
  loadingTeams.value = true;

  try {
    const list = await getUserTeams({ page: 1, limit: 100 });
    teams.value = list;
    console.log('汉化组列表加载成功:', teams.value);
  } catch (err) {
    console.error('获取汉化组列表失败:', err);
    teams.value = [];
    toastStore.show('获取汉化组列表失败', 'error');
  } finally {
    loadingTeams.value = false;
  }

  // （已移除示例用的补充数据）
}

// 点击用户头像 -> 加载用户参与项目
function selectUserProjects(): void {
  activeTeamId.value = null;
  // 留空：由 ProjectList 内部负责加载；这里仅标记 activeTeamId
}

// 打开项目详情
function handleOpenDetail(projectId: string): void {
  detailMode.value = 'detail';
  selectedProjectId.value = projectId;
  detailReady.value = false;
  // 如果当前未展开，则触发展开动画；若已展开，则立即标记为就绪，直接切换内容
  if (!detailOpen.value) {
    detailOpen.value = true;
  } else {
    detailReady.value = true;
  }
}

// 关闭项目详情
function handleCloseDetail(): void {
  detailOpen.value = false;
  detailReady.value = false;
}

// 右侧详情栏宽度动画结束后，标记为就绪再渲染复杂内容
function handleDetailTransitionEnd(e: TransitionEvent): void {
  if (e.propertyName !== 'width') return;
  if (!detailOpen.value) return;
  detailReady.value = true;
}

// 初始化加载（仅在拥有 moetran token 时进行）
onMounted(() => {
  // if (!tokenStore.moetranToken) {
  //   toastStore.show('尚未登录，无法载入仪表盘');
  //   return;
  // }
  loadUser();
  loadTeams();
  loadAnnouncements();
});

// ======================= 新过滤逻辑 =======================
// 来自 ProjectFilterBoard 的选项结构
// 筛选面板返回的单个筛选项
interface FilterOption {
  label: string;
  key: string;
  value: string;
}
const currentFilterOptions = ref<FilterOption[]>([]);

// 将 FilterOption[] 映射为后端可识别的 ProjectSearchFilters
// 后端定义字段：
// fuzzy_proj_name, translating_status, proofreading_status, typesetting_status, reviewing_status,
// is_published, member_ids, (扩展) project_set_id
interface PanelProjectSearchFilters extends ProjectSearchFilters {
  project_set_id?: string;
}

function mapPhaseTextToNumber(text: string): number | undefined {
  const raw = text.trim();
  if (!raw) return undefined;

  // 允许直接使用数字 0 / 1 / 2
  if (/^[0-2]$/.test(raw)) return Number(raw);

  const normalized = raw.toLowerCase();

  // 0 = 未开始 / 待开始
  if (['0', 'pending', '未开始', '待开始', '待处理', '未启动'].includes(normalized)) return 0;
  // 1 = 进行中
  if (['1', 'wip', '进行中', '进行', '处理中', '进行中中'].includes(normalized)) return 1;
  // 2 = 已完成
  if (['2', 'completed', '已完成', '完成', '结束', '已结束'].includes(normalized)) return 2;

  return undefined;
}

const currentSearchFilters = computed<PanelProjectSearchFilters | undefined>(() => {
  const opts = currentFilterOptions.value;

  if (!opts.length) return undefined;

  const filters: PanelProjectSearchFilters = {};
  const memberIds: string[] = [];

  for (const opt of opts) {
    const key = opt.key;
    const val = opt.value.trim();

    if (!val) continue;

    // 项目名称模糊匹配
    if (key === 'project') {
      filters.fuzzy_proj_name = val;
      continue;
    }

    // 项目集筛选（若后端支持，使用 project_set_id 字段）
    if (key === 'project-set') {
      filters.project_set_id = val;
      continue;
    }

    // 成员 / 指定角色成员（目前统一推入 member_ids，后端若需区分角色可扩展）
    if (key === 'member' || key.startsWith('member-')) {
      memberIds.push(val);
      continue;
    }

    // 各阶段状态（翻译 / 校对 / 嵌字 / 监修）
    if (key.endsWith('-status')) {
      // key 示例： translation-status / proofreading-status / typesetting-status / reviewing-status / publish-status
      const phaseBase = key.replace('-status', '');

      if (phaseBase === 'publish') {
        // 发布状态特殊：支持数字或文本输入 -> 2/"已完成"/"已发布" 视为已发布
        if (/^[0-2]$/.test(val)) {
          filters.is_published = Number(val) === 2;
        } else {
          filters.is_published = ['已发布', 'true', 'yes', 'published', '完成', '已完成'].includes(
            val.toLowerCase()
          );
        }
        continue;
      }

      const mapField: Record<string, keyof PanelProjectSearchFilters> = {
        translation: 'translating_status',
        proofreading: 'proofreading_status',
        typesetting: 'typesetting_status',
        reviewing: 'reviewing_status',
      };

      const targetField = mapField[phaseBase];
      if (targetField) {
        const num = mapPhaseTextToNumber(val);
        if (num !== undefined) (filters as PanelProjectSearchFilters)[targetField] = num;
      }
      continue;
    }
  }

  if (memberIds.length) filters.member_ids = memberIds;

  return filters;
});

function handleConfirmOptions(options: FilterOption[]) {
  currentFilterOptions.value = options;
  // 由 ProjectList 根据 currentSearchFilters 调用 IPC 搜索，无需本地过滤
}

// 最终传递给 ProjectList 的项目已由其内部管理；此处预留接口以备未来需要
// const displayProjects = computed(() =>
//   filteredProjects.value.length ? filteredProjects.value : rawProjects.value
// );

// 公告列表（示例数据）
interface Announcement {
  id: number;
  title: string;
  content: string;
  date: string;
}
const announcements = ref<Announcement[]>([]);
async function loadAnnouncements(): Promise<void> {
  // 示例 API
  await new Promise(r => setTimeout(r, 220));
  announcements.value = [
    {
      id: 1,
      title: '系统维护',
      content: '11/25 将进行短时维护，届时服务可能中断。',
      date: '2025-11-20',
    },
    { id: 2, title: '新功能上线', content: '新增成员筛选与项目模板支持。', date: '2025-11-10' },
    {
      id: 3,
      title: '社群活动',
      content: '本周五举办线上翻译交流会，欢迎报名。',
      date: '2025-11-08',
    },
  ];
}

// 打开新建项目视图（供 ProjectList 的 create 事件调用）
function handleOpenCreator(): void {
  detailMode.value = 'create';
  selectedProjectId.value = null;
  detailReady.value = false;

  if (!detailOpen.value) {
    detailOpen.value = true;
  } else {
    detailReady.value = true;
  }
}
</script>

<template>
  <div class="dashboard-root">
    <!-- 顶部栏 -->
    <header class="top-bar">
      <div class="top-bar__user" v-if="user">
        <span class="top-bar__name">{{ user.name }}</span>
        <span v-if="(user as any).signature" class="top-bar__signature">
          {{ (user as any).signature }}
        </span>
      </div>
      <div v-else class="top-bar__user top-bar__user--loading">
        <span v-if="loadingUser">载入用户...</span>
        <span v-else>未登录</span>
      </div>
    </header>

    <div class="content-layout">
      <!-- 侧边栏：始终是头像+名字纵向列表，收起靠裁剪隐藏名字 -->
      <aside class="teams-sidebar">
        <ul class="teams-list">
          <li class="team-item team-item--user" @click="selectUserProjects">
            <template v-if="user?.has_avatar && user?.avatar">
              <img
                class="team-item__avatar-img"
                :src="user.avatar"
                alt="user"
                @error="onUserAvatarError"
              />
            </template>
            <template v-else>
              <span class="team-item__avatar user-avatar">{{
                user?.name ? user.name.slice(0, 1) : '我'
              }}</span>
            </template>
            <span class="team-item__name">{{ user?.name || '我' }} 的项目</span>
          </li>
          <li
            v-for="team in teams"
            :key="team.id"
            class="team-item"
            @click="activeTeamId = team.id"
          >
            <template v-if="team.has_avatar && team.avatar">
              <img
                class="team-item__avatar-img"
                :src="team.avatar"
                :alt="team.name"
                @error="() => onTeamAvatarError(team.id)"
              />
            </template>
            <template v-else>
              <span class="team-item__avatar">{{ team.name.slice(0, 1) }}</span>
            </template>
            <span class="team-item__name">{{ team.name }} 的项目</span>
          </li>
          <li v-if="!loadingTeams && teams.length === 0" class="team-item team-item--empty">
            <span class="team-item__avatar">-</span>
            <span class="team-item__name">暂无汉化组</span>
          </li>
          <li v-if="loadingTeams" class="team-item team-item--loading">
            <span class="team-item__avatar">…</span>
            <span class="team-item__name">载入汉化组...</span>
          </li>
        </ul>
      </aside>

      <!-- 主体区域 -->
      <main class="projects-main">
        <div class="projects-top">
          <div class="projects-announcements">
            <div class="ann-header">
              <h3 class="ann-title">公告</h3>
            </div>
            <ul class="ann-list">
              <li v-for="a in announcements" :key="a.id" class="ann-item">
                <div class="ann-item__title">{{ a.title }}</div>
                <div class="ann-item__content">{{ a.content }}</div>
                <div class="ann-item__date">{{ a.date }}</div>
              </li>
              <li v-if="!announcements.length" class="ann-empty">暂无公告</li>
            </ul>
          </div>

          <div class="filter-panel-wrapper">
            <ProjectFilterBoard
              :team-id="activeTeamId ?? undefined"
              @confirmOptions="handleConfirmOptions"
            />
          </div>
        </div>
        <div class="projects-content" ref="projectsContainerRef">
          <div v-if="loadingProjects" class="placeholder">载入项目...</div>
          <div class="projects-scroll" v-else>
            <ProjectList
              :team-id="activeTeamId"
              :filters="currentSearchFilters"
              @open-detail="handleOpenDetail"
              @create="handleOpenCreator"
            />
          </div>
        </div>
      </main>

      <!-- 右侧项目详情伸缩侧栏 -->
      <aside
        class="detail-sidebar"
        :class="{ 'detail-sidebar--open': detailOpen }"
        :style="{ '--detail-sidebar-max-ratio': DETAIL_SIDEBAR_MAX_RATIO } as any"
        ref="detailSidebarRef"
        @transitionend="handleDetailTransitionEnd"
      >
        <div v-if="!detailReady" class="detail-sidebar__loading">
          <div class="detail-spinner"></div>
          <div class="detail-loading-text">加载项目详情...</div>
        </div>
        <ProjectDetailView
          v-else-if="detailMode === 'detail' && selectedProjectId !== null"
          :project-id="selectedProjectId"
          @close="handleCloseDetail"
          @open-translator="() => {}"
        />
        <ProjectCreatorView
          v-else-if="detailMode === 'create'"
          :team-id="activeTeamId || undefined"
          @close="handleCloseDetail"
        />
      </aside>
    </div>
  </div>
</template>
<style scoped>
/* 根布局 */
.dashboard-root {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: linear-gradient(180deg, #f4f9ff 0%, #ffffff 70%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
}

/* 顶部栏 */
.top-bar {
  height: 54px;
  display: flex;
  align-items: center;
  padding: 0 28px;
  background: rgba(255, 255, 255, 0.92);
  border-bottom: 1px solid rgba(150, 180, 210, 0.35);
  box-shadow: 0 6px 18px rgba(140, 180, 230, 0.18);
}
.top-bar__user {
  font-size: 14px;
  font-weight: 600;
  display: flex;
  gap: 14px;
  align-items: baseline;
}
.top-bar__name {
  color: #203b56;
}
.top-bar__signature {
  font-size: 12px;
  color: #4a6a85;
}
.top-bar__user--loading {
  opacity: 0.7;
}

.content-layout {
  flex: 1;
  display: flex;
  flex-direction: row;
  padding: 16px 24px 28px;
  gap: 22px;
}

/* 侧边栏：单层头像+名字列表，通过宽度+裁剪表现收起/展开 */
.teams-sidebar {
  width: 80px; /* 收起时宽度，仅头像区域 */
  flex: 0 0 auto;
  display: block;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 16px;
  box-shadow: 0 8px 28px rgba(140, 180, 230, 0.18);
  height: calc(100vh - 54px - 44px);
  overflow: hidden; /* 隐藏溢出部分（名字在收起时被裁剪） */
  transition: width 0.28s ease;
}
.teams-sidebar:hover {
  width: clamp(260px, 26vw, 360px);
}

/* 当侧栏收起（未 hover）时，隐藏名字，防止文字首字母在收起状态下溢出 */
.teams-sidebar:not(:hover) .team-item__name {
  max-width: 0;
  opacity: 0;
  transform: translateX(-4px);
  transition:
    max-width 0.18s ease,
    opacity 0.18s ease,
    transform 0.18s ease;
  overflow: hidden;
  white-space: nowrap;
  pointer-events: none;
}

/* 确保列表项在 flex 收缩时不会导致子元素溢出 */
.team-item {
  min-width: 0;
}

.teams-list {
  list-style: none;
  margin: 0;
  padding: 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.team-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 12px;
  background: #fff;
  font-size: 13px;
  cursor: pointer;
  transition:
    box-shadow 0.16s ease,
    transform 0.16s ease,
    background-color 0.16s ease;
}
.team-item:hover {
  box-shadow: 0 6px 18px rgba(136, 190, 247, 0.25);
  transform: translateY(-2px);
  background: #f5f8ff;
}

.team-item__avatar {
  flex: 0 0 34px;
  width: 34px;
  min-width: 34px;
  height: 28px;
  border-radius: 8px;
  background: #eef6ff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  color: #2a4b66;
  border: 1px solid rgba(150, 180, 210, 0.5);
}
.team-item--user .team-item__avatar {
  background: linear-gradient(135deg, #cde6ff, #e6f4ff);
}

/* Avatar image styles */
.team-item__avatar-img {
  width: 34px;
  height: 28px;
  min-width: 34px;
  min-height: 28px;
  border-radius: 8px;
  object-fit: cover;
  display: block;
  box-sizing: border-box;
  align-self: center;
}

.top-bar__avatar-img {
  width: 34px;
  height: 34px;
  min-width: 34px;
  min-height: 34px;
  border-radius: 50%;
  object-fit: cover;
  margin-right: 10px;
  box-sizing: border-box;
}

.top-bar__avatar {
  display: inline-flex;
  width: 34px;
  min-width: 34px;
  height: 34px;
  min-height: 34px;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: #eef6ff;
  color: #2a4b66;
  font-weight: 600;
  margin-right: 10px;
  box-sizing: border-box;
}

/* Ensure avatar containers don't shrink when parent gets narrow */
.team-item__avatar,
.team-item__avatar-img,
.top-bar__avatar,
.top-bar__avatar-img {
  flex-shrink: 0;
}

.team-item__name {
  flex: 1 1 auto;
  min-width: 0; /* 让 ellipsis 生效 */
  font-weight: 600;
  color: #1f2e43;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-item--empty,
.team-item--loading {
  opacity: 0.8;
}

.projects-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.3);
  border-radius: 18px;
  padding: 20px 22px 26px;
  box-sizing: border-box;
  box-shadow: 0 10px 34px rgba(140, 180, 230, 0.22);
  height: calc(100vh - 54px - 44px);
  min-width: 0; /* 防止被右侧栏挤压时产生水平抖动 */
}
.projects-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.projects-title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}
.projects-top {
  display: flex;
  gap: 18px;
  /* stretch children so left (announcements) and right (filter panel)
     share the same top and bottom edges */
  align-items: stretch;
}
.projects-announcements {
  flex: 1;
  background: #fff;
  border: 1px solid rgba(170, 190, 215, 0.85);
  border-radius: 12px;
  padding: 12px;
  box-sizing: border-box;
  min-height: 84px;
}
.ann-header {
  margin-bottom: 8px;
}
.ann-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: #203650;
}
.ann-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.ann-item {
  padding: 8px;
  border-radius: 8px;
  background: #fbfdff;
  border: 1px solid rgba(230, 240, 250, 0.9);
}
.ann-item__title {
  font-weight: 600;
  font-size: 13px;
  color: #214b66;
}
.ann-item__content {
  font-size: 13px;
  color: #48657a;
  margin-top: 4px;
}
.ann-item__date {
  font-size: 12px;
  color: #7a8b99;
  margin-top: 6px;
}
.ann-empty {
  text-align: center;
  color: #6b7c91;
  padding: 10px 0;
}
/* 让原有的 filter-panel-wrapper 在并排时占右半 */
.filter-panel-wrapper {
  /* keep the wrapper lightweight to avoid double borders; the child
     (ProjectFilterBoard) will receive the visible card styling so only
     a single border is shown and typography/height can match announcements */
  flex: 1;
  display: flex;
  flex-direction: column;
}
.filter-toggle {
  border: 1px solid rgba(118, 184, 255, 0.7);
  background: #f1f7ff;
  color: #2f5a8f;
  width: 38px;
  height: 34px;
  border-radius: 10px;
  cursor: pointer;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease;
}
.filter-toggle:hover {
  box-shadow: 0 6px 16px rgba(136, 190, 247, 0.25);
  transform: translateY(-2px);
}

/* 顶部右侧只有筛选按钮，创建入口在 ProjectList 内部 */

/* 筛选面板折叠动效 */
.filter-panel-wrapper {
  overflow: hidden;
  transition:
    max-height 0.3s ease,
    opacity 0.25s ease,
    margin-top 0.3s ease;
}
/* apply the card styling to the direct child so we don't produce a
   double border (ProjectFilterBoard itself already renders a bordered
   card); also ensure it fills the wrapper and uses matching font size */
.filter-panel-wrapper > * {
  flex: 1 1 auto;
  min-height: 84px;
  box-sizing: border-box;
  background: #fff;
  border: 1px solid rgba(170, 190, 215, 0.85);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  font-size: 13px;
}

/* remove the filter-board's default top margin and slightly adjust
   its top padding so its inner content aligns with announcements */
.filter-panel-wrapper > .filter-board {
  margin-top: 0;
  padding-top: 12px;
}
.filter-panel-fade-enter-from,
.filter-panel-fade-leave-to {
  opacity: 0;
  max-height: 0;
  margin-top: 0;
}
.filter-panel-fade-enter-active {
  max-height: 400px;
  margin-top: 12px;
}
.filter-panel-fade-enter-to {
  max-height: 400px;
  margin-top: 12px;
  opacity: 1;
}
.filter-panel-fade-leave-active {
  max-height: 400px;
  margin-top: 12px;
}
.filter-panel-fade-leave-from {
  max-height: 400px;
  margin-top: 12px;
  opacity: 1;
}
.projects-content {
  margin-top: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.projects-scroll {
  flex: 1;
  overflow: auto;
  padding-right: 6px; /* 给滚动条留出些许空间，防止内容紧贴 */
}
.placeholder {
  font-size: 13px;
  color: #6b859d;
}

/* 右侧详情伸缩侧栏，与左侧类似的宽度动画 */
.detail-sidebar {
  width: 0;
  flex: 0 0 auto;
  overflow: hidden;
  height: calc(100vh - 54px - 44px);
  background: #ffffff;
  border-radius: 18px;
  border: 1px solid rgba(150, 180, 210, 0.35);
  box-shadow: 0 10px 34px rgba(140, 180, 230, 0.22);
  box-sizing: border-box;
  transition:
    width 0.28s ease,
    opacity 0.22s ease;
  opacity: 0;
}

.detail-sidebar--open {
  /* 使用比例控制最大宽度，可通过 DETAIL_SIDEBAR_MAX_RATIO 调整 */
  width: calc(var(--detail-sidebar-max-ratio, 0.5) * 100%);
  opacity: 1;
}

.detail-sidebar__loading {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  color: #4a5f7a;
  font-size: 13px;
}

.detail-spinner {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 3px solid rgba(160, 190, 230, 0.35);
  border-top-color: rgba(118, 184, 255, 0.95);
  animation: detail-spin 0.8s linear infinite;
}

.detail-loading-text {
  opacity: 0.85;
}

@keyframes detail-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
