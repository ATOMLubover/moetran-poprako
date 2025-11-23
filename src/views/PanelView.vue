<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useToastStore } from '../stores/toast';
import { getUserInfo } from '../ipc/user';
import { getUserTeams } from '../ipc/team';
import { getUserProjects } from '../ipc/project';
import type { ResUser } from '../api/model/user';
import type { ResTeam } from '../api/model/team';
import ProjectList from '../components/ProjectList.vue';
import type { ResProject } from '../api/model/project';
import type { ProjectBasicInfo, PhaseChip, WorkPhase } from '../api/model/displayProject';
// 新增过滤面板组件（综合筛选逻辑）——暂使用 mock，实现与示例类似功能
import ProjectFilterBoard from '../components/ProjectFilterBoard.vue';
import ProjectDetailView from '../views/ProjectDetailView.vue';
import ProjectCreatorView from '../views/ProjectCreatorView.vue';

// 用户信息
const user = ref<ResUser | null>(null);

// 汉化组列表
const teams = ref<ResTeam[]>([]);
const teamsLimit = ref<number>(100); // 暂不分页

// 主体“当前展示”的项目列表（可能是：用户项目 / 某项目集项目）
// 基础原始数据（未过滤）与过滤后数据分离
// 使用共享的展示类型
const rawProjects = ref<ProjectBasicInfo[]>([]);
const filteredProjects = ref<ProjectBasicInfo[]>([]);
const projectsLimit = ref<number>(50);

// 右侧详情栏相关状态
const selectedProjectId = ref<number | null>(null);
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

// 载入用户信息
async function loadUser(): Promise<void> {
  loadingUser.value = true;

  try {
    // TODO: mock
    // user.value = await getUserInfo();
  } catch (err) {
    toastStore.show('获取用户信息失败');
  }

  loadingUser.value = false;
}

// 载入汉化组列表
async function loadTeams(): Promise<void> {
  loadingTeams.value = true;

  try {
    // TODO：mock
    teams.value = [];
    toastStore.show('获取汉化组列表失败（使用 mock）');
  } catch (err) {
    // IPC 异常时使用 mock 兜底
    teams.value = [];
    toastStore.show('获取汉化组列表失败（使用 mock）');
  }

  loadingTeams.value = false;
  // 总是补充 mock 汉化组以便测试展开动效（确保至少 8 个）
  const base = teams.value.length;
  for (let i = base; i < 8; i++) {
    teams.value.push({
      id: String(9000 + i),
      name: 'Mock组' + (i + 1),
      avatar: '',
      has_avatar: false,
    });
  }
}

// ========== Mock 辅助：生成阶段集 ==========
function createPhaseSet(seed: number): PhaseChip[] {
  const phases: WorkPhase[] = ['translate', 'proof', 'typeset', 'review', 'publish'];
  const labelMap: Record<WorkPhase, string> = {
    translate: '翻译',
    proof: '校对',
    typeset: '嵌字',
    review: '监修',
    publish: '发布',
  };

  return phases.map((phase, i) => {
    const rotate = (seed + i) % 5;
    let status: PhaseChip['status'] = 'unset';
    if (rotate === 1) status = 'pending';
    else if (rotate === 2) status = 'wip';
    else if (rotate === 3) status = 'completed';

    return { phase, status, label: labelMap[phase] } satisfies PhaseChip;
  });
}

// 加载“我参与的项目”（真实应调用 getUserProjects）
async function loadUserProjects(): Promise<void> {
  loadingProjects.value = true;
  try {
    // TODO: mock
    rawProjects.value = [
      { id: 101, index: 1, author: '作者A', title: '第一个项目标题', phases: createPhaseSet(3) },
      { id: 102, index: 2, author: '作者B', title: '第二个项目标题', phases: createPhaseSet(6) },
      { id: 103, index: 3, author: '作者C', title: '第三个项目标题', phases: createPhaseSet(9) },
    ];
    // const apiRes = await getUserProjects({ page: 1, limit: projectsLimit.value });
    // // 将 ResProject 映射为 DisplayProject（此处假设字段存在，可根据真实结构调整）
    // rawProjects.value = apiRes.map((p, idx) => {
    //   const rp = p as ResProject & Partial<{ author: string; title: string }>;
    //   return {
    //     id: Number(rp.id),
    //     index: idx + 1,
    //     author: rp.author ?? '未知作者',
    //     title: rp.title ?? '未命名项目',
    //     phases: createPhaseSet(idx * 7 + 3),
    //   } satisfies ProjectBasicInfo;
    // });
  } catch (err) {
    // 若接口异常，用 mock 数据兜底
    rawProjects.value = [
      { id: 101, index: 1, author: '作者A', title: '第一个项目标题', phases: createPhaseSet(3) },
      { id: 102, index: 2, author: '作者B', title: '第二个项目标题', phases: createPhaseSet(6) },
      { id: 103, index: 3, author: '作者C', title: '第三个项目标题', phases: createPhaseSet(9) },
    ];
    toastStore.show('获取用户项目失败（使用 mock）');
  }
  loadingProjects.value = false;
  applyFilters();
}

// 分页相关逻辑暂未实现，保留接口占位

// 点击用户头像 -> 加载用户参与项目
function selectUserProjects(): void {
  loadUserProjects();
}

// 打开项目详情
function handleOpenDetail(projectId: number): void {
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
  loadUserProjects(); // 初始加载用户项目
});

// ======================= 新过滤逻辑 =======================
// 来自 ProjectFilterBoard 的选项结构
interface FilterOption {
  label: string;
  key: string;
  value: string;
}
const currentFilterOptions = ref<FilterOption[]>([]);

// 应用过滤（非常简化，只演示关键匹配，真实场景需与后端参数协议对接）
function applyFilters() {
  let base = [...rawProjects.value];
  for (const opt of currentFilterOptions.value) {
    if (opt.key === 'project-set') {
      base = base.filter(
        p => String(p.projectSetId || '') === opt.value || p.title.includes(opt.value)
      );
    } else if (opt.key === 'project') {
      const v = opt.value;
      base = base.filter(
        p => p.title.includes(v) || p.author.includes(v) || String(p.id).includes(v)
      );
    } else if (opt.key === 'member') {
      base = base.filter(p => p.author.includes(opt.value));
    } else if (opt.key.endsWith('-status')) {
      // 状态：翻译/校对/嵌字/监修/发布，对应 phases 中 phase
      const phaseKey = opt.key.replace('-status', '').replace('typesetting', 'typeset');
      base = base.filter(p => p.phases.some(ph => ph.phase.startsWith(phaseKey))); // 简化处理
    }
  }
  filteredProjects.value = base;
}

function handleConfirmOptions(options: FilterOption[]) {
  currentFilterOptions.value = options;
  applyFilters();
}

// 最终传递给 ProjectList 的项目（若有过滤则用过滤结果）
const displayProjects = computed(() =>
  filteredProjects.value.length ? filteredProjects.value : rawProjects.value
);

// 筛选控制板折叠状态（默认收起）
const filterOpen = ref(false);

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
        <span
          v-if="(user as ResUser & { signature?: string }).signature"
          class="top-bar__signature"
          >{{ (user as ResUser & { signature?: string }).signature }}</span
        >
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
            <span class="team-item__avatar user-avatar">我</span>
            <span class="team-item__name">{{ user?.name || '我的项目' }}</span>
          </li>
          <li v-for="team in teams" :key="team.id" class="team-item" @click="loadUserProjects()">
            <span class="team-item__avatar">{{ team.name.slice(0, 1) }}</span>
            <span class="team-item__name">{{ team.name }}</span>
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
        <div class="projects-header">
          <h1 class="projects-title">项目列表</h1>
          <button
            class="filter-toggle"
            @click="filterOpen = !filterOpen"
            :aria-expanded="filterOpen"
            title="筛选"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <circle cx="10" cy="10" r="7" stroke="currentColor" stroke-width="2" />
              <path
                d="M15 15L21 21"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
        <transition name="filter-panel-fade">
          <div v-show="filterOpen" class="filter-panel-wrapper">
            <ProjectFilterBoard @confirmOptions="handleConfirmOptions" />
          </div>
        </transition>
        <div class="projects-content">
          <div v-if="loadingProjects" class="placeholder">载入项目...</div>
          <div class="projects-scroll" v-else>
            <ProjectList
              :projects="displayProjects"
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
        <ProjectCreatorView v-else-if="detailMode === 'create'" @close="handleCloseDetail" />
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
  font-size: 18px;
  font-weight: 600;
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
