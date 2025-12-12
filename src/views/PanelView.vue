<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import type { ComponentPublicInstance } from 'vue';
import type { ProjectSearchFilters } from '../ipc/project';
import type { FilterOption } from '../api/model/filterOption';
import { useToastStore } from '../stores/toast';
import { useUserStore } from '../stores/user';
import { useRouterStore } from '../stores/router';
import { useCacheStore } from '../stores/cache';
import { getUserInfo } from '../ipc/user';
import { getUserTeams } from '../ipc/team';
import { getAllCachedProjects } from '../ipc/image_cache';
import type { ResUser } from '../api/model/user';
import type { ResTeam } from '../api/model/team';
import type { ResMember } from '../api/model/member';
import ProjectList from '../components/ProjectList.vue';
import AssignmentList from '../components/AssignmentList.vue';
import MemberList from '../components/MemberList.vue';
import LoadingCircle from '../components/LoadingCircle.vue';
// 使用共享的基本项目信息类型仅在本地过滤场景；当前不直接使用
import ProjectFilterBoard from '../components/ProjectFilterBoard.vue';
import AssignmentFilterBoard from '../components/AssignmentFilterBoard.vue';
import ProjectCreatorView from '../views/ProjectCreatorView.vue';
import ProjectModifierView from '../views/ProjectModifierView.vue';
import ProjectDetailView from '../views/ProjectDetailView.vue';
import ProjectSetCreatorView from '../views/ProjectSetCreatorView.vue';
import CachedProjectsModal from '../components/CachedProjectsModal.vue';

// 用户信息 (local ref kept for template compatibility)
const user = ref<ResUser | null>(null);
const userStore = useUserStore();

// 缓存项目管理
const showCachedModal = ref(false);

// 汉化组列表
const teams = ref<ResTeam[]>([]);
// 当前选中的团队（用于成员筛选等场景；null 代表“仅看我的项目”）
const activeTeamId = ref<string | null>(null);

// 视图模式：项目列表或派活列表或成员列表
const viewMode = ref<'projects' | 'assignments' | 'members'>('projects');

// 主体“当前展示”的项目列表目前由 ProjectList 自行通过 IPC 拉取。
// 这里预留 rawProjects / filteredProjects 以便后续若改回父组件统筹过滤时使用。
// 现阶段先注释掉，避免未使用变量的类型检查错误。
// const rawProjects = ref<ProjectBasicInfo[]>([]);
// const filteredProjects = ref<ProjectBasicInfo[]>([]);

// 右侧详情栏相关状态
// 侧边栏模式：创建 / 详情 / 修改
const detailMode = ref<'detail' | 'create' | 'modifier'>('create');
const detailOpen = ref(false);
const detailReady = ref(false);

// 项目修改时需要的数据
const selectedProjectId = ref<string | null>(null);
const selectedProjectTitle = ref<string>('');
const selectedProjectProjsetName = ref<string | null>(null);
const selectedProjectProjsetIndex = ref<number | null>(null);
const selectedProjectTotalMarkers = ref<number | null>(null);
const selectedProjectTotalTranslated = ref<number | null>(null);
const selectedProjectTotalChecked = ref<number | null>(null);
const selectedProjectTranslators = ref<string[]>([]);
const selectedProjectProofreaders = ref<string[]>([]);
const selectedProjectLetterers = ref<string[]>([]);
const selectedProjectReviewers = ref<string[]>([]);
const selectedProjectPrincipals = ref<string[]>([]);
const selectedProjectMembers = ref<ResMember[] | undefined>(undefined);
// 私有类型：panel 中仅用于标记/传递 role 值（未知结构；避免使用不安全的 any 类型）
interface _PanelSelectedRole {
  [key: string]: unknown;
}

const selectedProjectRole = ref<_PanelSelectedRole | null>(null);
const selectedProjectHasPoprako = ref<boolean | undefined>(undefined);
const selectedProjectTranslatingStatus = ref<number | null>(null);
const selectedProjectProofreadingStatus = ref<number | null>(null);
const selectedProjectTypesettingStatus = ref<number | null>(null);
const selectedProjectReviewingStatus = ref<number | null>(null);
const selectedProjectIsPublished = ref<boolean>(false);
const selectedProjectTeamId = ref<string>('');
const selectedProjectTeamName = ref<string>('');

// 项目集创建弹窗状态
const projsetCreatorOpen = ref(false);
// 私有类型：暴露给父组件的筛选板实例方法（只需 reloadProjsets）
interface _PanelFilterBoardRef extends ComponentPublicInstance {
  reloadProjsets?: () => void;
}

const filterBoardRef = ref<_PanelFilterBoardRef | null>(null);

// 加载状态
const loadingUser = ref<boolean>(false);
const loadingTeams = ref<boolean>(false);
const loadingProjects = ref<boolean>(false);

// 依赖的 store
const toastStore = useToastStore();
const routerStore = useRouterStore();
const cacheStore = useCacheStore();

// 扫描并初始化项目文件缓存状态
async function scanProjectFileCaches(): Promise<void> {
  try {
    console.log('[PanelView] Scanning project file caches...');

    const allCached = await getAllCachedProjects();

    console.log('[PanelView] Found cached projects', { count: allCached.length, allCached });

    for (const meta of allCached) {
      // status === 'completed' 表示该项目已成功缓存
      const hasCached = meta.status === 'completed';

      cacheStore.setProjectFileCacheState(meta.projectId, hasCached);

      console.log('[PanelView] Set cache state for project', {
        projectId: meta.projectId,
        hasCached,
      });
    }

    console.log('[PanelView] Project file cache scan complete');
  } catch (err) {
    console.error('[PanelView] Failed to scan project file caches', err);
  }
}

// 载入用户信息
async function loadUser(): Promise<void> {
  loadingUser.value = true;

  try {
    user.value = await getUserInfo();
    console.log('用户信息加载成功:', user.value);
    // persist to Pinia store for other components to consult
    userStore.setUser(user.value);
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

    // If backend includes is_admin in the team objects, record it into userStore
    try {
      (list as ResTeam[]).forEach(t => {
        if (t && typeof t.id === 'string' && Object.prototype.hasOwnProperty.call(t, 'isAdmin')) {
          // note: API model may not include this field in TS types, so we defensive-check at runtime
          const tt = t as unknown as { isAdmin?: unknown };
          userStore.setAdminForTeam(t.id, Boolean(tt.isAdmin));
        }
      });
    } catch (e) {
      // ignore
    }
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

// 处理视图切换
function handleViewChange(view: 'projects' | 'assignments' | 'members'): void {
  viewMode.value = view;
}

// 重新登录：导航到登录视图
function handleRelogin(): void {
  routerStore.navigateToLogin();
}

// 选择某个汉化组：同时刷新当前用户在该组的成员信息（含 is_admin）
async function onSelectTeam(teamId: string): Promise<void> {
  activeTeamId.value = teamId;

  try {
    const { getMemberInfo } = await import('../ipc/member');
    const info = await getMemberInfo(teamId);
    // eslint-disable-next-line no-console
    console.log('[PanelView] member info for team', teamId, info);
    userStore.setAdminForTeam(teamId, info.isAdmin === true);
  } catch (err) {
    // 若获取失败（例如 404 或 PopRaKo 未启动），仅记录日志并保持默认非管理员
    // eslint-disable-next-line no-console
    console.warn('[PanelView] failed to load member info for team', teamId, err);
  }
}

// 打开项目详情（由 ProjectList 传入 enriched + 成员信息）
function handleOpenDetail(payload: {
  id: string;
  title: string;
  projsetName: string | null;
  projsetIndex: number | null;
  principals?: string[];
  totalMarkers: number | null;
  totalTranslated: number | null;
  totalChecked: number | null;
  translatingStatus: number | null;
  proofreadingStatus: number | null;
  typesettingStatus: number | null;
  reviewingStatus: number | null;
  translators: string[];
  proofreaders: string[];
  letterers: string[];
  reviewers: string[];
  members?: ResMember[];
  isPublished?: boolean;
  hasPoprako?: boolean;
  role?: _PanelSelectedRole | null;
  teamId?: string;
  teamName?: string;
}): void {
  // 在 PanelView 中打开全屏详情视图
  detailMode.value = 'detail'; // 设置为 detail 模式
  selectedProjectId.value = payload.id;
  selectedProjectTitle.value = payload.title;
  selectedProjectProjsetName.value = payload.projsetName;
  selectedProjectProjsetIndex.value = payload.projsetIndex;
  selectedProjectTotalMarkers.value = payload.totalMarkers;
  selectedProjectTotalTranslated.value = payload.totalTranslated;
  selectedProjectTotalChecked.value = payload.totalChecked;
  selectedProjectTranslatingStatus.value = payload.translatingStatus;
  selectedProjectProofreadingStatus.value = payload.proofreadingStatus;
  selectedProjectTypesettingStatus.value = payload.typesettingStatus;
  selectedProjectReviewingStatus.value = payload.reviewingStatus;
  selectedProjectTranslators.value = payload.translators;
  selectedProjectProofreaders.value = payload.proofreaders;
  selectedProjectLetterers.value = payload.letterers;
  selectedProjectReviewers.value = payload.reviewers;
  selectedProjectPrincipals.value = payload.principals ?? [];
  selectedProjectMembers.value = payload.members ?? undefined;
  selectedProjectRole.value = payload.role ?? null;
  selectedProjectHasPoprako.value = payload.hasPoprako ?? undefined;
  selectedProjectIsPublished.value = payload.isPublished ?? false;
  // 直接使用 payload 中的 teamId，不再依赖响应式的 activeTeamId
  selectedProjectTeamId.value = payload.teamId ?? '';
  selectedProjectTeamName.value = payload.teamName ?? '';
  detailReady.value = false;
  detailOpen.value = true;
  // 直接标记为就绪，全屏显示
  detailReady.value = true;
}

// 关闭项目详情/创建/修改侧边栏
function handleCloseDetail(): void {
  detailOpen.value = false;
  detailReady.value = false;
  selectedProjectMembers.value = undefined;
  selectedProjectRole.value = null;
  selectedProjectHasPoprako.value = undefined;
}

// 打开项目集创建弹窗
function handleOpenProjsetCreator(): void {
  if (!activeTeamId.value) return;
  projsetCreatorOpen.value = true;
}

// 关闭项目集创建弹窗
function handleCloseProjsetCreator(): void {
  projsetCreatorOpen.value = false;
}

// 项目集创建成功后刷新筛选板的项目集列表
function handleProjsetCreated(): void {
  if (filterBoardRef.value?.reloadProjsets) {
    filterBoardRef.value.reloadProjsets();
  }
  projsetCreatorOpen.value = false;
}

// 工具箱：打开嵌字用字体池（占位）
function handleOpenFontPool(): void {
  console.log('[PanelView] Open font pool (placeholder)');
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
  // 扫描项目文件缓存状态
  scanProjectFileCaches();
});

// 当团队改变时，如果切换到“我”（null），强制切换到项目列表
watch(
  () => activeTeamId.value,
  newTeamId => {
    if (newTeamId === null && (viewMode.value === 'assignments' || viewMode.value === 'members')) {
      viewMode.value = 'projects';
    }
  }
);

// ======================= 新过滤逻辑 =======================
// 来自 ProjectFilterBoard 的选项结构
const currentFilterOptions = ref<FilterOption[]>([]);

// 追踪搜索状态，用于禁用筛选控制板
const isSearching = ref(false);

// 处理添加筛选项（来自 ProjectFilterBoard）
function handleAddFilterOption(option: FilterOption) {
  const exists = currentFilterOptions.value.find(
    opt => opt.key === option.key && opt.value === option.value
  );

  if (!exists) {
    currentFilterOptions.value.push(option);
  }
}

// 处理移除筛选项（来自 ProjectFilterBoard）
function handleRemoveFilterOption(option: FilterOption) {
  const index = currentFilterOptions.value.findIndex(
    opt => opt.key === option.key && opt.value === option.value
  );

  if (index !== -1) {
    currentFilterOptions.value.splice(index, 1);
  }
}

// 处理清空所有筛选项（来自 ProjectFilterBoard）
function handleClearFilters() {
  currentFilterOptions.value = [];
}

// 将 FilterOption[] 映射为后端可识别的 ProjectSearchFilters
// 后端定义字段：
// fuzzy_proj_name, translating_status, proofreading_status, typesetting_status, reviewing_status,
// is_published, member_ids, (扩展) project_set_id
interface PanelProjectSearchFilters extends ProjectSearchFilters {
  projectSetIds?: string[];
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
      filters.fuzzyProjName = val;
      continue;
    }

    // 项目集筛选（支持多个 project set）
    if (key === 'project-set') {
      if (!filters.projectSetIds) filters.projectSetIds = [];
      (filters.projectSetIds as string[]).push(val);
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
          filters.isPublished = Number(val) === 2;
        } else {
          filters.isPublished = ['已发布', 'true', 'yes', 'published', '完成', '已完成'].includes(
            val.toLowerCase()
          );
        }
        continue;
      }

      const mapField: Record<string, keyof PanelProjectSearchFilters> = {
        translation: 'translatingStatus',
        proofreading: 'proofreadingStatus',
        typesetting: 'typesettingStatus',
        reviewing: 'reviewingStatus',
      };

      const targetField = mapField[phaseBase];
      if (targetField) {
        const num = mapPhaseTextToNumber(val);
        if (num !== undefined) (filters as PanelProjectSearchFilters)[targetField] = num;
      }
      continue;
    }

    // 时间筛选（time-start）
    if (key === 'time-start') {
      const timestamp = Number(val);
      if (!isNaN(timestamp) && timestamp > 0) {
        filters.timeStart = timestamp;
      }
      continue;
    }
  }

  if (memberIds.length) filters.memberIds = memberIds;

  return filters;
});

// Debug: log when the computed search filters change (helps trace v-model -> computed -> prop flow)
watch(
  () => currentSearchFilters.value,
  filters => {
    if (filters !== undefined) {
      console.log('[PanelView] currentSearchFilters changed:', JSON.parse(JSON.stringify(filters)));
    } else {
      console.log('[PanelView] currentSearchFilters changed: undefined (no filters)');
    }

    // 筛选条件变化时标记为搜索中，短暂延时后解除
    isSearching.value = true;

    setTimeout(() => {
      isSearching.value = false;
    }, 500);
  },
  { deep: true }
);

// ======================= 派活筛选逻辑 =======================
// 派活筛选条件接口（仅支持时间筛选）
interface AssignmentSearchFilters {
  timeStart?: number;
}

// 来自 AssignmentFilterBoard 的选项结构
const currentAssignmentFilterOptions = ref<FilterOption[]>([]);

// 处理添加筛选项（来自 AssignmentFilterBoard）
function handleAddAssignmentFilterOption(option: FilterOption) {
  console.log('[PanelView] handleAddAssignmentFilterOption called with:', option);

  const exists = currentAssignmentFilterOptions.value.find(
    opt => opt.key === option.key && opt.value === option.value
  );

  if (!exists) {
    currentAssignmentFilterOptions.value.push(option);
    console.log(
      '[PanelView] Added assignment filter option, current options:',
      currentAssignmentFilterOptions.value
    );
  }
}

// 处理移除筛选项（来自 AssignmentFilterBoard）
function handleRemoveAssignmentFilterOption(option: FilterOption) {
  const index = currentAssignmentFilterOptions.value.findIndex(
    opt => opt.key === option.key && opt.value === option.value
  );

  if (index !== -1) {
    currentAssignmentFilterOptions.value.splice(index, 1);
  }
}

// 处理清空所有筛选项（来自 AssignmentFilterBoard）
function handleClearAssignmentFilters() {
  currentAssignmentFilterOptions.value = [];
}

// 将 FilterOption[] 映射为后端可识别的 AssignmentSearchFilters
const currentAssignmentSearchFilters = computed<AssignmentSearchFilters | undefined>(() => {
  if (!currentAssignmentFilterOptions.value.length) return undefined;

  const filters: AssignmentSearchFilters = {};

  for (const opt of currentAssignmentFilterOptions.value) {
    const { key, value: val } = opt;

    // 时间筛选（time-start）
    if (key === 'time-start') {
      const timestamp = Number(val);
      if (!isNaN(timestamp) && timestamp > 0) {
        filters.timeStart = timestamp;
      }
      continue;
    }
  }

  return filters;
});

// Debug: log when the computed assignment search filters change
watch(
  () => currentAssignmentSearchFilters.value,
  filters => {
    if (filters !== undefined) {
      console.log(
        '[PanelView] currentAssignmentSearchFilters changed:',
        JSON.parse(JSON.stringify(filters))
      );
    } else {
      console.log('[PanelView] currentAssignmentSearchFilters changed: undefined (no filters)');
    }
  },
  { deep: true }
);

// 筛选即时生效：不再需要手动应用流程（确认按钮已移除）

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
  detailOpen.value = true;
  // 直接标记为就绪，全屏显示
  detailReady.value = true;
}

// 打开修改项目视图（由 ProjectDetailView 触发）
function handleOpenModifier(): void {
  detailMode.value = 'modifier';
  detailReady.value = false;
  detailOpen.value = true;
  // 直接标记为就绪，全屏显示
  detailReady.value = true;
}

// Cancel from modifier: back to detail view
function handleModifierBack(): void {
  detailMode.value = 'detail';
  detailReady.value = false;
  detailReady.value = true;
}
</script>

<template>
  <div class="dashboard-root">
    <!-- 顶部栏 已移除（按要求） -->

    <div class="content-layout">
      <!-- 左侧：汉化组列表 -->
      <aside class="teams-sidebar">
        <ul class="teams-list">
          <li
            :class="[
              'team-item',
              'team-item--user',
              { 'team-item--selected': activeTeamId === null },
            ]"
            @click="selectUserProjects"
          >
            <span class="team-item__avatar user-avatar">{{
              user?.name ? user.name.slice(0, 1) : '我'
            }}</span>
            <span class="team-item__name">{{ user?.name || '我' }} 的项目</span>
          </li>
          <li
            v-for="team in teams"
            :key="team.id"
            :class="['team-item', { 'team-item--selected': activeTeamId === team.id }]"
            @click="onSelectTeam(team.id)"
          >
            <span class="team-item__avatar">{{ team.name.slice(0, 1) }}</span>
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
          <li class="team-item team-item--refresh" @click="loadTeams">
            <span class="team-item__avatar">⟳</span>
            <span class="team-item__name">刷新汉化组</span>
          </li>
        </ul>
        <div class="teams-sidebar-footer">
          <button class="relogin-button" @click="handleRelogin">重新登录</button>
        </div>
      </aside>

      <!-- 中间：项目列表/派活主区域 -->
      <main class="projects-main">
        <div class="projects-content" ref="projectsContainerRef">
          <div v-if="loadingProjects" class="placeholder">载入项目...</div>
          <div class="projects-scroll" v-else>
            <ProjectList
              v-if="viewMode === 'projects'"
              :team-id="activeTeamId"
              :filters="currentSearchFilters"
              :current-view="viewMode"
              @open-detail="handleOpenDetail"
              @create="handleOpenCreator"
              @view-change="handleViewChange"
              @create-projset="handleOpenProjsetCreator"
            />
            <AssignmentList
              v-else-if="viewMode === 'assignments'"
              :team-id="activeTeamId"
              :current-view="viewMode"
              :filters="currentAssignmentSearchFilters"
              @view-change="handleViewChange"
            />
            <MemberList
              v-else-if="viewMode === 'members'"
              :team-id="activeTeamId"
              :current-view="viewMode"
              @view-change="handleViewChange"
            />
          </div>
        </div>
      </main>

      <!-- 右侧：PopRaKo 筛选控制板（占用全部宽度的 23%） -->
      <aside class="right-column">
        <div class="filter-panel-wrapper">
          <!-- 项目筛选板（仅在项目列表视图显示） -->
          <ProjectFilterBoard
            v-if="viewMode === 'projects'"
            :team-id="activeTeamId ?? undefined"
            :is-searching="isSearching"
            @add-option="handleAddFilterOption"
            @remove-option="handleRemoveFilterOption"
            @clear-all="handleClearFilters"
            ref="filterBoardRef"
          />

          <!-- 派活筛选板（仅在派活列表视图显示） -->
          <AssignmentFilterBoard
            v-else-if="viewMode === 'assignments'"
            :is-searching="isSearching"
            @add-option="handleAddAssignmentFilterOption"
            @remove-option="handleRemoveAssignmentFilterOption"
            @clear-all="handleClearAssignmentFilters"
          />

          <!-- 工具箱：与 teams-list 类似的组织方式，紧贴在筛选板下方 -->
          <div class="toolbox">
            <div class="toolbox-header">工具箱</div>

            <ul class="toolbox-list teams-list">
              <li class="team-item team-item--cache" @click="showCachedModal = true">
                <span class="team-item__avatar cache-avatar">✔</span>
                <span class="team-item__name">缓存项目</span>
              </li>

              <li class="team-item" @click="handleOpenFontPool">
                <span class="team-item__avatar">字</span>
                <span class="team-item__name">嵌字用字体池</span>
              </li>
            </ul>
          </div>
        </div>
      </aside>

      <!-- 全屏项目详情/创建/修改视图 -->
      <div v-if="detailOpen" class="detail-fullscreen">
        <div class="detail-fullscreen__container">
          <div v-if="!detailReady" class="detail-fullscreen__loading">
            <LoadingCircle />
          </div>
          <ProjectDetailView
            v-else-if="detailMode === 'detail' && selectedProjectId !== null"
            :project-id="selectedProjectId"
            :title="selectedProjectTitle"
            :projset-name="selectedProjectProjsetName"
            :projset-index="selectedProjectProjsetIndex"
            :principals="selectedProjectPrincipals"
            :total-markers="selectedProjectTotalMarkers"
            :total-translated="selectedProjectTotalTranslated"
            :total-checked="selectedProjectTotalChecked"
            :translating-status="selectedProjectTranslatingStatus"
            :proofreading-status="selectedProjectProofreadingStatus"
            :typesetting-status="selectedProjectTypesettingStatus"
            :reviewing-status="selectedProjectReviewingStatus"
            :translators="selectedProjectTranslators"
            :proofreaders="selectedProjectProofreaders"
            :letterers="selectedProjectLetterers"
            :reviewers="selectedProjectReviewers"
            :members="selectedProjectMembers"
            :is-published="selectedProjectIsPublished"
            :role="selectedProjectRole"
            :has-poprako="selectedProjectHasPoprako"
            :team-id="selectedProjectTeamId"
            :team-name="selectedProjectTeamName"
            @close="handleCloseDetail"
            @open-modifier="handleOpenModifier"
          />

          <ProjectCreatorView
            v-else-if="detailMode === 'create'"
            :team-id="activeTeamId || undefined"
            @close="handleCloseDetail"
          />
          <ProjectModifierView
            v-else-if="detailMode === 'modifier' && selectedProjectId !== null"
            :project-id="selectedProjectId"
            :project-name="selectedProjectTitle"
            :project-description="''"
            :team-id="selectedProjectTeamId"
            :projset-name="selectedProjectProjsetName"
            :members="selectedProjectMembers"
            :translating-status="selectedProjectTranslatingStatus"
            :proofreading-status="selectedProjectProofreadingStatus"
            :typesetting-status="selectedProjectTypesettingStatus"
            :reviewing-status="selectedProjectReviewingStatus"
            :is-published="selectedProjectIsPublished"
            @close="handleCloseDetail"
            @back="handleModifierBack"
          />
        </div>
      </div>

      <!-- 缓存项目管理悬浮窗 -->
      <CachedProjectsModal v-if="showCachedModal" @close="showCachedModal = false" />

      <!-- 项目集创建弹窗 -->
      <div v-if="projsetCreatorOpen" class="projset-overlay">
        <div class="projset-creator-modal">
          <ProjectSetCreatorView
            :team-id="activeTeamId"
            @close="handleCloseProjsetCreator"
            @created="handleProjsetCreated"
          />
        </div>
      </div>
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
  padding: 16px 24px 10px;
  gap: 22px;
}

/* 侧边栏：单层头像+名字列表，始终展开显示 */
.teams-sidebar {
  width: 200px; /* 减少宽度 */
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 16px;
  box-shadow: 0 8px 28px rgba(140, 180, 230, 0.18);
  overflow: hidden; /* 隐藏溢出部分 */
  min-height: 0; /* allow children to flex/scroll properly */
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
  flex: 1 1 auto; /* take remaining space so footer stays pinned to bottom */
  overflow: auto;
}

/* 侧栏底部区域，用于操作按钮（如重新登录） */
.teams-sidebar-footer {
  padding: 10px;
  border-top: 1px solid rgba(150, 180, 210, 0.08);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.96), rgba(250, 250, 250, 0.98));
  display: flex;
  justify-content: center;
}
.relogin-button {
  background: transparent;
  color: #2f6fb2; /* subtle blue */
  border: 1px solid rgba(47, 111, 178, 0.16);
  padding: 8px 12px;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  font-size: 13px;
}
.relogin-button:hover {
  background: rgba(47, 111, 178, 0.06);
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

/* 选中项样式：绿色边框与淡绿色背景 */
.team-item--selected {
  border: 1px solid rgba(46, 204, 113, 0.9);
  background: linear-gradient(180deg, #f6fff6 0%, #ecfff0 100%);
  box-shadow: 0 8px 22px rgba(46, 204, 113, 0.06);
  transform: none; /* keep selected steady */
}

.team-item--selected .team-item__avatar {
  background: linear-gradient(135deg, #eaffea, #e0ffe0);
  border-color: rgba(46, 204, 113, 0.45);
  color: #1f5d35;
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

/* 刷新按钮：不加粗、字体缩小并使用斜体 */
.team-item.team-item--refresh .team-item__name {
  font-weight: 400;
  font-size: 12px;
  font-style: italic;
}

.team-item--empty,
.team-item--loading {
  opacity: 0.8;
}

.team-item--cache {
  border-top: 1px solid rgba(150, 180, 210, 0.5);
  padding-top: 10px;
  margin-bottom: 8px;
}

.projects-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.3);
  border-radius: 18px;
  padding: 20px 22px 6px; /* 减少底部 padding，从 26px 改为 6px */
  box-sizing: border-box;
  box-shadow: 0 10px 34px rgba(140, 180, 230, 0.22);
  min-width: 0; /* 防止被右侧栏挤压时产生水平抖动 */
}
/* 右侧第三列：占据全部宽度的 23% */
.right-column {
  flex: 0 0 23%;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Ensure the filter panel inside the right column fills its container */
.right-column .filter-panel-wrapper,
.right-column .filter-panel-wrapper > * {
  width: 100%;
  box-sizing: border-box;
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
  padding: 6px 8px;
  border-radius: 8px;
  background: #fbfdff;
  border: 1px solid rgba(230, 240, 250, 0.9);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ann-item__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.ann-item__title {
  font-weight: 600;
  font-size: 13px;
  color: #214b66;
  /* do not allow the title to push the date out; truncate if too long */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
}
.ann-item__date {
  font-size: 12px;
  color: #7a8b99;
  flex: 0 0 auto;
  margin-left: 12px;
  white-space: nowrap;
}
.ann-item__content {
  font-size: 13px;
  color: #48657a;
  /* single-line truncation to compress vertical space */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
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
/* keep the wrapper lightweight so the child component's card
   styling (border/background/shadow) remains authoritative. The
   wrapper only handles sizing and layout; avoid forcing a background
   or border onto the child. */
.filter-panel-wrapper > * {
  /* Match the visual style of the main projects panel: same border,
     radius and shadow so the filter card reads as the same system.
     Keep flex disabled so the child sizes to its content. */
  flex: 0 0 auto;
  min-height: 84px;
  box-sizing: border-box;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.3); /* match .projects-main */
  border-radius: 18px; /* match .projects-main */
  padding: 12px;
  display: flex;
  flex-direction: column;
  font-size: 13px;
  box-shadow: 0 10px 34px rgba(140, 180, 230, 0.22); /* match .projects-main */
}

/* When multiple child cards are stacked inside the wrapper, remove the
   gap by flattening adjacent border radii so they visually join. */
.filter-panel-wrapper > * + * {
  margin-top: 0;
  border-top-left-radius: 0;
  border-top-right-radius: 0;
}
.filter-panel-wrapper > *:first-child {
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
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
  margin-top: 2px;
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

/* 隐藏滚动条的 hack 技巧 */
.projects-scroll::-webkit-scrollbar {
  display: none;
}
.projects-scroll {
  -ms-overflow-style: none; /* IE and Edge */
  scrollbar-width: none; /* Firefox */
}
.placeholder {
  font-size: 13px;
  color: #6b859d;
}

/* 全屏详情视图 */
.detail-fullscreen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.detail-fullscreen__container {
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 18px;
  box-shadow: 0 10px 34px rgba(140, 180, 230, 0.22);
  width: 70%;
  height: 80%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.detail-fullscreen__loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 项目集创建弹窗样式 */
.projset-overlay {
  position: fixed;
  inset: 0;
  background: rgba(10, 20, 40, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}

.projset-creator-modal {
  width: 600px;
  max-width: calc(100% - 40px);
  max-height: calc(100vh - 80px);
  overflow: hidden;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(30, 60, 100, 0.4);
}

/* 工具箱样式：与侧栏列表风格一致 */
.toolbox {
  /* styled as a card child inside filter-panel-wrapper; outer wrapper
     child-selector will handle borders/radii when stacked */
  margin: 0;
  overflow: hidden;
}
.toolbox-header {
  padding: 8px 10px;
  font-size: 13px;
  font-weight: 700;
  color: #203650;
}
.toolbox-list {
  /* mirror teams-list organization but keep compact height */
  list-style: none;
  margin: 0;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 220px;
  overflow: auto;
}

.toolbox .team-item {
  padding: 6px 8px;
  border-radius: 10px;
}

/* Remove the explicit top border/padding for cache item when it's shown
   inside the toolbox so it visually joins the filter card above. */
.toolbox .team-item--cache {
  border-top: none;
  padding-top: 6px;
  margin-bottom: 0;
}
</style>
