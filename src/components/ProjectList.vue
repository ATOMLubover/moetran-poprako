<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useToastStore } from '../stores/toast';
import { useUserStore } from '../stores/user';
import type { ProjectBasicInfo, PhaseStatus, PhaseChip } from '../api/model/displayProject';
import {
  getUserProjectsEnriched,
  searchUserProjectsEnriched,
  searchTeamProjectsEnriched,
  getTeamProjectsEnriched,
} from '../ipc/project';
import type { ProjectSearchFilters } from '../ipc/project';
import type { ResProjectEnriched } from '../api/model/project';
import type { ResMember } from '../api/model/member';

// 使用共享类型定义（见 src/api/model/displayProject.ts）

// 私有类型：用于 open-detail 事件的负载，避免匿名结构体
interface _ProjectListOpenDetailPayload {
  id: string;
  title: string;
  projsetName: string | null;
  projsetIndex: number | null;
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
  principals?: string[];
  members?: ResMember[];
  isPublished?: boolean;
  hasPoprako?: boolean;
  role?: _ProjectRole | null;
  teamId?: string;
  teamName?: string;
}

// 私有类型：通用 role passthrough
interface _ProjectRole {
  [key: string]: unknown;
}

// 组件事件：打开详情 / 创建项目
const emit = defineEmits<{
  (e: 'open-detail', payload: _ProjectListOpenDetailPayload): void;
  (e: 'create'): void;
  (e: 'view-change', view: 'projects' | 'assignments'): void;
  (e: 'create-projset'): void;
}>();

const props = defineProps<{
  // 当前激活的汉化组 id，null/undefined 表示"仅看我自己的项目"
  teamId?: string | null;
  // 来自 PanelView 的筛选条件；空对象或 undefined 表示不启用筛选
  filters?: ProjectSearchFilters | undefined;
  // 当此布尔值切换时，触发列表刷新（用于确认/清空筛选）
  shouldApplyFilters?: boolean;
  // 当前视图：'projects' 或 'assignments'
  currentView?: 'projects' | 'assignments';
}>();

const userStore = useUserStore();

const canCreate = computed(() => {
  // allow create only when a team is selected and the user is admin for that team
  return !!props.teamId && userStore.isAdminFor(props.teamId);
});

// 视图模式：项目列表或派活列表
const viewMode = computed(() => props.currentView ?? 'projects');

// 切换视图
function switchView(view: 'projects' | 'assignments'): void {
  emit('view-change', view);
}

// 派活列表在无 teamId 时应禁用
const canViewAssignments = computed(() => !!props.teamId);

// 内部项目列表与加载状态
type ProjectListItem = ProjectBasicInfo & { hasPoprako?: boolean } & {
  // enriched 统计数据，供 ProjectDetail 使用
  totalMarkers?: number;
  totalTranslated?: number;
  totalChecked?: number;
  translatingStatusRaw?: number | null;
  proofreadingStatusRaw?: number | null;
  typesettingStatusRaw?: number | null;
  reviewingStatusRaw?: number | null;
  translators?: string[];
  proofreaders?: string[];
  letterers?: string[];
  reviewers?: string[];
  isPublished?: boolean;
  principals?: string[];
  projectSetName?: string | null;
  members?: ResMember[];
  role?: _ProjectRole | null;
  // 汉化组信息（来自 ResProjectEnriched 的 team 字段）
  teamId?: string;
  teamName?: string;
};

// 本地完整项目缓存（所有已拉取的数据）
const allProjects = ref<ProjectListItem[]>([]);
// 当前显示的项目列表（根据屏幕高度裁剪后的）
const innerProjects = ref<ProjectListItem[]>([]);
// 当前显示列表的起始索引（在 allProjects 中的位置）
const currentStartIndex = ref(0);
const isLoading = ref(false);
const listContainerRef = ref<HTMLElement | null>(null);
// resize debounce timer and listener ref
const resizeTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const resizeListener = ref<((this: Window, ev: UIEvent) => void) | null>(null);
// 服务端一次最多拉取多少条
const serverLimit = 10;
// 服务端分页状态（用于判断是否还有更多数据可拉取）
const serverPage = ref(1);
const lastFetchCount = ref(0);
// 是否还有下一页（服务端是否还有更多数据）
const hasMoreFromServer = computed(() => lastFetchCount.value === serverLimit);

// 上一页：尝试从本地缓存向前翻页
function goPrevPage(): void {
  if (currentStartIndex.value <= 0) return;

  // 计算当前显示的数量，作为一“页”的大小
  const currentPageSize = innerProjects.value.length;
  // 向前移动一页
  currentStartIndex.value = Math.max(0, currentStartIndex.value - currentPageSize);

  void clampAndDisplay();
}

// 下一页：先检查本地是否有足够数据，不足时请求更多
async function goNextPage(): Promise<void> {
  const currentPageSize = innerProjects.value.length;
  const nextStartIndex = currentStartIndex.value + currentPageSize;

  // 检查本地缓存是否有足够的数据显示下一页
  // 如果本地剩余数据少于 currentPageSize 且服务端还有更多数据，则请求
  const remainingLocal = allProjects.value.length - nextStartIndex;

  if (remainingLocal < currentPageSize && hasMoreFromServer.value) {
    // 需要从服务端拉取更多数据
    await fetchMoreFromServer();
  }

  // 检查是否还有数据可以显示
  if (nextStartIndex >= allProjects.value.length) return;

  currentStartIndex.value = nextStartIndex;
  void clampAndDisplay();
}

function refreshList(): void {
  // 刷新时清空缓存，重新从第一页开始
  allProjects.value = [];
  currentStartIndex.value = 0;
  serverPage.value = 1;
  void fetchAndClamp();
}

// 从服务端拉取更多数据（追加到 allProjects）
async function fetchMoreFromServer(): Promise<void> {
  if (isLoading.value) return;

  console.log('[ProjectList] fetchMoreFromServer: fetching page', serverPage.value + 1);
  isLoading.value = true;

  try {
    serverPage.value += 1;
    let apiRes: ResProjectEnriched[] = [];

    const hasFilters = !!(props.filters && Object.keys(props.filters).length > 0);

    if (props.teamId) {
      if (hasFilters) {
        apiRes = await searchTeamProjectsEnriched({
          team_id: props.teamId as string,
          ...props.filters,
          page: serverPage.value,
          limit: serverLimit,
        });
      } else {
        apiRes = await getTeamProjectsEnriched({
          teamId: props.teamId as string,
          page: serverPage.value,
          limit: serverLimit,
        });
      }
    } else {
      if (hasFilters) {
        apiRes = await searchUserProjectsEnriched({
          ...props.filters,
          page: serverPage.value,
          limit: serverLimit,
        });
      } else {
        apiRes = await getUserProjectsEnriched({
          page: serverPage.value,
          limit: serverLimit,
        });
      }
    }

    const newItems = mapEnrichedToBasic(apiRes);
    lastFetchCount.value = apiRes.length;

    // 追加到本地缓存
    allProjects.value = [...allProjects.value, ...newItems];

    console.log(
      '[ProjectList] fetchMoreFromServer: added',
      newItems.length,
      'items, total:',
      allProjects.value.length
    );
  } catch (err) {
    console.error('[ProjectList] fetchMoreFromServer failed:', err);
    const toastStore = useToastStore();
    toastStore.show('获取项目失败，请稍后重试');
  } finally {
    isLoading.value = false;
  }
}

// 点击详情
function handleOpenDetail(item: ProjectListItem): void {
  emit('open-detail', {
    id: item.id,
    title: item.title,
    projsetName: item.projectSetName ?? null,
    projsetIndex: item.projectSetIndex ?? null,
    totalMarkers: item.totalMarkers ?? null,
    totalTranslated: item.totalTranslated ?? null,
    totalChecked: item.totalChecked ?? null,
    translatingStatus: item.translatingStatusRaw ?? null,
    proofreadingStatus: item.proofreadingStatusRaw ?? null,
    typesettingStatus: item.typesettingStatusRaw ?? null,
    reviewingStatus: item.reviewingStatusRaw ?? null,
    translators: item.translators ?? [],
    proofreaders: item.proofreaders ?? [],
    letterers: item.letterers ?? [],
    reviewers: item.reviewers ?? [],
    principals: item.principals ?? [],
    members: item.members ?? [],
    isPublished: item.isPublished ?? false,
    hasPoprako: item.hasPoprako ?? false,
    role: item.role ?? null,
    teamId: item.teamId ?? undefined,
    teamName: item.teamName ?? undefined,
  });
}

// 从 allProjects 中裁剪并显示当前页面（根据屏幕高度）
function clampAndDisplay(): void {
  void nextTick().then(() => {
    requestAnimationFrame(() => {
      const container = listContainerRef.value;
      if (!container) {
        console.log('[ProjectList] clampAndDisplay: container not mounted');
        return;
      }

      if (allProjects.value.length === 0) {
        innerProjects.value = [];
        return;
      }

      // 从 currentStartIndex 开始，取尽可能多的项，直到超出屏幕高度
      const availableItems = allProjects.value.slice(currentStartIndex.value);

      // 先尝试显示所有可用项
      innerProjects.value = availableItems;

      // 等待 DOM 更新后再裁剪
      void nextTick().then(() => {
        requestAnimationFrame(() => {
          const scroll = container.closest('.projects-scroll') as HTMLElement | null;
          const host = scroll ?? container;
          const items = host.querySelectorAll('.project-list__item');

          if (!items.length) {
            console.log('[ProjectList] clampAndDisplay: no items rendered');
            return;
          }

          let visibleCount = 0;

          for (let i = 0; i < items.length; i++) {
            const itemRect = (items[i] as HTMLElement).getBoundingClientRect();
            const bottomWithPadding = itemRect.bottom + 20;

            if (bottomWithPadding > window.innerHeight) {
              break;
            }

            visibleCount++;
          }

          // 至少显示一项
          const finalCount = Math.max(1, visibleCount);

          console.log(
            '[ProjectList] clampAndDisplay: visible items:',
            finalCount,
            'of',
            availableItems.length
          );

          if (innerProjects.value.length > finalCount) {
            innerProjects.value = availableItems.slice(0, finalCount);
          }
        });
      });
    });
  });
}

// 创建项目：交给父组件打开右侧创建表单视图
function handleCreateProject(): void {
  emit('create');
}

// Chip 样式计算（基于 status 返回类名）
function chipClass(phase: PhaseChip): string {
  const base = 'project-list__chip';

  const map: Record<PhaseStatus, string> = {
    unset: 'project-list__chip--unset',
    pending: 'project-list__chip--pending',
    wip: 'project-list__chip--wip',
    completed: 'project-list__chip--completed',
  };

  return `${base} ${map[phase.status]}`;
}

// 最终展示数据：始终使用内部拉取的 innerProjects
const displayProjects = computed<ProjectListItem[]>(() => innerProjects.value);

// 分页按钮状态
const canGoPrev = computed(() => currentStartIndex.value > 0);
const canGoNext = computed(() => {
  const currentPageSize = innerProjects.value.length;
  const nextStartIndex = currentStartIndex.value + currentPageSize;
  // 如果本地还有数据，或者服务端还有更多，则可以下一页
  return nextStartIndex < allProjects.value.length || hasMoreFromServer.value;
});

// 当前页码显示（基于 currentStartIndex 和 innerProjects.length 计算）
const currentPageNumber = computed(() => {
  if (innerProjects.value.length === 0) return 1;
  return Math.floor(currentStartIndex.value / innerProjects.value.length) + 1;
});

// 将 ResProjectEnriched 转为列表展示 DTO
function mapEnrichedToBasic(apiRes: ResProjectEnriched[]): ProjectListItem[] {
  return apiRes.map(p => {
    const labelMap: Record<PhaseChip['phase'], string> = {
      translate: '翻译',
      proof: '校对',
      typeset: '嵌字',
      review: '监修',
      publish: '发布',
    };

    // Helper: map numeric status (0/1/2) to PhaseStatus
    const numToPhaseStatus = (n: number | undefined): PhaseStatus => {
      if (n === 1) return 'wip';
      if (n === 2) return 'completed';
      // treat undefined or 0 as unset
      return 'unset';
    };

    let phases: PhaseChip[] = [];

    if (!p.hasPoprako) {
      // Native projects (no PopRaKo): show a single gray tag
      phases = [
        {
          phase: 'translate',
          status: 'unset',
          label: '尨译原生项目',
        } as PhaseChip,
      ];
    } else {
      // PopRaKo-backed project: show PopRaKo tag (template already renders a small green "PopRaKo" tag),
      // and display real phase statuses from the enriched DTO.
      phases = [
        {
          phase: 'translate',
          status: numToPhaseStatus(p.translatingStatus),
          label: labelMap.translate,
        } as PhaseChip,
        {
          phase: 'proof',
          status: numToPhaseStatus(p.proofreadingStatus),
          label: labelMap.proof,
        } as PhaseChip,
        {
          phase: 'typeset',
          status: numToPhaseStatus(p.typesettingStatus),
          label: labelMap.typeset,
        } as PhaseChip,
        {
          phase: 'review',
          status: numToPhaseStatus(p.reviewingStatus),
          label: labelMap.review,
        } as PhaseChip,
        {
          phase: 'publish',
          status: p.isPublished ? 'completed' : 'unset',
          label: labelMap.publish,
        } as PhaseChip,
      ];
    }

    const base: ProjectListItem = {
      // 后端 id 是 UUID，保持为字符串
      id: p.id,
      title: p.name,
      projectSetId: p.projectSet?.id,
      projectSetSerial: p.projsetSerial,
      projectSetIndex: p.projsetIndex,
      projectSetName: p.projectSet?.name ?? null,
      hasPoprako: p.hasPoprako,
      totalMarkers: p.sourceCount,
      totalTranslated: p.translatedSourceCount,
      totalChecked: p.checkedSourceCount,
      translatingStatusRaw: p.translatingStatus ?? null,
      proofreadingStatusRaw: p.proofreadingStatus ?? null,
      typesettingStatusRaw: p.typesettingStatus ?? null,
      reviewingStatusRaw: p.reviewingStatus ?? null,
      isPublished: p.isPublished ?? false,
      principals: p.principals ?? [],
      members: (p.members ?? []) as ResMember[],
      role: p.role ?? null,
      teamId: p.team?.id ?? undefined,
      teamName: p.team?.name ?? undefined,
      phases,
    };

    return base;
  });
}

// 初始化或重置时调用：从服务端拉取第一页数据
async function fetchAndClamp(): Promise<void> {
  console.log(
    '[ProjectList] fetchAndClamp called with teamId:',
    props.teamId,
    'filters:',
    props.filters
  );
  isLoading.value = true;

  try {
    console.log(
      '[ProjectList] fetchAndClamp: requesting first page, limit =',
      serverLimit,
      'teamId =',
      props.teamId
    );

    let apiRes: ResProjectEnriched[] = [];

    const hasFilters = !!(props.filters && Object.keys(props.filters).length > 0);
    console.log('[ProjectList] hasFilters:', hasFilters, 'teamId:', props.teamId);

    if (props.teamId) {
      if (hasFilters) {
        apiRes = await searchTeamProjectsEnriched({
          team_id: props.teamId as string,
          ...props.filters,
          page: 1,
          limit: serverLimit,
        });
      } else {
        apiRes = await getTeamProjectsEnriched({
          teamId: props.teamId as string,
          page: 1,
          limit: serverLimit,
        });
      }
    } else {
      if (hasFilters) {
        apiRes = await searchUserProjectsEnriched({
          ...props.filters,
          page: 1,
          limit: serverLimit,
        });
      } else {
        apiRes = await getUserProjectsEnriched({ page: 1, limit: serverLimit });
      }
    }

    const all = mapEnrichedToBasic(apiRes);
    lastFetchCount.value = apiRes.length;

    // 重置本地缓存
    allProjects.value = all;
    currentStartIndex.value = 0;
    serverPage.value = 1;

    console.log('[ProjectList] fetchAndClamp: loaded', all.length, 'items');

    // 裁剪并显示
    clampAndDisplay();
  } catch (err) {
    console.error('[ProjectList] fetchAndClamp failed:', err);
    allProjects.value = [];
    innerProjects.value = [];
    lastFetchCount.value = 0;

    const toastStore = useToastStore();
    toastStore.show('获取项目失败，请稍后重试');
  } finally {
    console.log('[ProjectList] fetchAndClamp finished');
    isLoading.value = false;
  }
}

onMounted(() => {
  requestAnimationFrame(() => {
    void fetchAndClamp();
  });
  // 当窗口尺寸变化时，debounce 后重新裁剪（保持当前起始位置）
  const onResize = () => {
    if (resizeTimer.value) clearTimeout(resizeTimer.value);
    resizeTimer.value = setTimeout(() => {
      requestAnimationFrame(() => {
        // resize 时不改变 currentStartIndex，只重新裁剪显示
        clampAndDisplay();
      });
    }, 150);
  };

  window.addEventListener('resize', onResize);
  // store listener so we can remove it on unmount
  resizeListener.value = onResize;
});

// 当 filters 变化时，清空缓存并重新拉取
watch(
  () => props.filters,
  () => {
    allProjects.value = [];
    currentStartIndex.value = 0;
    serverPage.value = 1;
    requestAnimationFrame(() => {
      void fetchAndClamp();
    });
  },
  { deep: true }
);

// 当 shouldApplyFilters 切换时，触发刷新（确认/清空筛选）
watch(
  () => props.shouldApplyFilters,
  () => {
    console.log('[ProjectList] shouldApplyFilters changed, calling fetchAndClamp');
    allProjects.value = [];
    currentStartIndex.value = 0;
    serverPage.value = 1;
    requestAnimationFrame(() => {
      void fetchAndClamp();
    });
  }
);

// 当 teamId 变化时也需重新拉取
watch(
  () => props.teamId,
  (newVal, oldVal) => {
    console.log('[ProjectList] teamId changed:', oldVal, '->', newVal);
    allProjects.value = [];
    currentStartIndex.value = 0;
    serverPage.value = 1;
    requestAnimationFrame(() => {
      void fetchAndClamp();
    });
  }
);

onBeforeUnmount(() => {
  // 清理 resize 监听器
  if (resizeListener.value) {
    window.removeEventListener('resize', resizeListener.value);
    resizeListener.value = null;
  }
  if (resizeTimer.value) {
    clearTimeout(resizeTimer.value);
    resizeTimer.value = null;
  }
});
</script>

<template>
  <section class="project-list">
    <header class="project-list__header">
      <!-- 标题改为切换按钮 -->
      <div class="project-list__view-toggle">
        <button
          type="button"
          class="view-toggle-btn"
          :class="{ 'view-toggle-btn--active': viewMode === 'projects' }"
          @click="switchView('projects')"
        >
          项目列表
        </button>
        <button
          type="button"
          class="view-toggle-btn"
          :class="{ 'view-toggle-btn--active': viewMode === 'assignments' }"
          @click="switchView('assignments')"
          :disabled="!canViewAssignments"
        >
          派活列表
        </button>
      </div>
      <div
        class="project-list__header-actions"
        :class="{ 'project-list__header-actions--locked': !canCreate }"
      >
        <button
          type="button"
          class="icon-btn"
          @click="refreshList"
          :disabled="isLoading"
          title="刷新"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="23 4 23 10 17 10"></polyline>
            <polyline points="1 20 1 14 7 14"></polyline>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"></path>
            <path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"></path>
          </svg>
        </button>
        <button
          type="button"
          class="project-list__publish"
          @click="$emit('create-projset')"
          :disabled="isLoading || !canCreate"
        >
          创建项目集
        </button>
        <button
          type="button"
          class="project-list__create"
          @click="handleCreateProject"
          :disabled="isLoading || !canCreate"
        >
          创建新项目
        </button>
        <!-- <span v-if="!canCreate" class="project-list__locked-note">🔒 仅团队管理员可创建</span> -->
        <div class="pagination-controls">
          <button
            type="button"
            class="icon-btn"
            @click="goPrevPage"
            :disabled="isLoading || !canGoPrev"
            title="上一页"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
          </button>
          <span class="page-indicator">第 {{ currentPageNumber }} 页</span>
          <button
            type="button"
            class="icon-btn"
            @click="goNextPage"
            :disabled="isLoading || !canGoNext"
            title="下一页"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
        </div>
      </div>
    </header>

    <div class="project-list__content" ref="listContainerRef">
      <ul class="project-list__items" v-if="displayProjects.length > 0">
        <li
          v-for="item in displayProjects"
          :key="String(item.id)"
          class="project-list__item"
          :class="{ 'project-list__item--poprako': item.hasPoprako }"
        >
          <div class="project-list__item-main">
            <h3 class="project-list__item-title">
              {{ item.title }}
              <span v-if="item.hasPoprako" class="project-list__tag-poprako">PopRaKo</span>
            </h3>
            <div class="project-list__chips">
              <span v-for="phase in item.phases" :key="phase.phase" :class="chipClass(phase)">
                {{ phase.label }}
              </span>
            </div>
          </div>
          <div class="project-list__actions">
            <button
              v-if="!item.isPublished"
              type="button"
              class="project-list__detail-btn"
              @click="handleOpenDetail(item)"
            >
              详情
            </button>
            <button v-else type="button" class="project-list__detail-btn" disabled>已发布</button>
          </div>
        </li>
      </ul>
      <div v-else-if="isLoading" class="project-list__loading">加载中...</div>
      <div v-else class="project-list__empty">暂无项目</div>
    </div>
  </section>
</template>

<style scoped>
.project-list {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 16px 30px 10px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.92);
  color: #28405c;
  min-width: 0;
  width: 100%;
  box-sizing: border-box;
}

.project-list__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.project-list__header-actions {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}
.pagination-controls {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-left: 8px;
  font-size: 12px;
  color: #2a4f7a;
}
.page-indicator {
  font-weight: 600;
}
.icon-btn {
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  padding: 4px 6px;
  border-radius: 8px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition:
    background 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.15s ease;
}
.icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.icon-btn:not(:disabled):hover {
  background: #eef6ff;
  box-shadow: 0 4px 14px rgba(118, 184, 255, 0.25);
  transform: translateY(-1px);
}

/* When the current user cannot create projects for the selected team,
   show the actions area with a muted/locked appearance while keeping layout. */
.project-list__header-actions--locked {
  opacity: 0.95;
}

.project-list__header-actions--locked .project-list__create,
.project-list__header-actions--locked .project-list__publish {
  background: linear-gradient(135deg, #f6f8fa, #fbfdff);
  color: #7a8796;
  border-color: rgba(200, 208, 218, 0.6);
  box-shadow: none;
}

.project-list__locked-note {
  margin-left: 6px;
  font-size: 13px;
  color: #6d7a8a;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.project-list__title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: #1f2e43;
}

.project-list__view-toggle {
  display: flex;
  gap: 0;
  border-radius: 10px;
  overflow: hidden;
  border: 2px solid rgba(118, 184, 255, 0.25);
  background: #f4f9ff;
}

.view-toggle-btn {
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 600;
  border: none;
  background: transparent;
  color: #2f5a8f;
  cursor: pointer;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.view-toggle-btn:not(:last-child) {
  border-right: 2px solid rgba(118, 184, 255, 0.15);
}

.view-toggle-btn:hover:not(:disabled):not(.view-toggle-btn--active) {
  background: #eef6ff;
}

.view-toggle-btn--active {
  background: linear-gradient(135deg, #5ba3e0, #6db4f0);
  color: #fff;
}

.view-toggle-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  color: #7a8796;
}

.project-list__create,
.project-list__publish {
  border-radius: 10px;
  padding: 8px 12px;
  font-size: 14px;
  font-weight: 600;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  cursor: pointer;
  box-shadow: 0 6px 16px rgba(118, 184, 255, 0.06);
  transition:
    transform 0.14s ease,
    box-shadow 0.14s ease,
    background 0.12s ease;
}

.project-list__create:hover:not(:disabled),
.project-list__publish:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.12);
  background: #eef6ff;
}

.project-list__create:disabled,
.project-list__publish:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
}

.project-list__create:not(:disabled):hover,
.project-list__publish:not(:disabled):hover {
  transform: translateY(-2px);
  box-shadow: 0 14px 30px rgba(118, 184, 255, 0.42);
}

.project-list__publish {
  padding-inline: 14px;
}

.project-list__content {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
}

.project-list__items {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px; /* 更紧凑的间距 */
}

.project-list__item {
  display: grid;
  grid-template-columns: 1fr auto; /* 左侧内容自适应，右侧动作固定 */
  gap: 12px; /* 减少左右内容间距 */
  padding: 10px 12px; /* 缩小内边距以降低高度 */
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.98);
  border: 1px solid rgba(210, 220, 235, 0.65);
  box-shadow: 0 6px 12px rgba(132, 166, 212, 0.08);
  transition:
    box-shadow 0.14s ease,
    transform 0.14s ease,
    border-color 0.14s ease;
}

.project-list__item--poprako {
  border-color: rgba(140, 205, 170, 0.9);
  box-shadow: 0 12px 26px rgba(120, 190, 160, 0.26);
}

.project-list__item:hover {
  box-shadow: 0 16px 32px rgba(132, 166, 212, 0.22);
  border-color: rgba(118, 184, 255, 0.85);
  transform: translateY(-2px);
}

.project-list__item-main {
  display: flex;
  flex-direction: column;
}

.project-list__item-title {
  margin: 0;
  font-size: 14px; /* 稍微小一点 */
  font-weight: 600;
  color: #294061;
  letter-spacing: 0.3px;
  /* 强制单行显示标题，与右侧动作在同一行 */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.project-list__chips {
  margin-top: 8px; /* 确保 chips 在标题下方单独一行 */
}

.project-list__tag-poprako {
  display: inline-block;
  margin-right: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  background: rgba(210, 244, 225, 0.96);
  color: #1e6042;
  border: 1px solid rgba(140, 205, 170, 0.9);
  vertical-align: middle;
}

.project-list__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 8px;
  align-items: center;
}

.project-list__chip {
  padding: 3px 8px 4px; /* 更紧凑的 chip */
  border-radius: 999px;
  font-size: 11px;
  letter-spacing: 0.2px;
  font-weight: 600;
  line-height: 1;
  border: 1px solid transparent;
  background: rgba(240, 246, 255, 0.9);
  color: #2a4f7a;
  user-select: none;
}

.project-list__chip--unset {
  background: rgba(234, 238, 244, 0.9);
  color: #6d7a8a;
  border-color: rgba(200, 208, 218, 0.6);
}

.project-list__chip--pending {
  background: rgba(255, 238, 210, 0.9);
  color: #7a4f20;
  border-color: rgba(250, 203, 143, 0.7);
}

.project-list__chip--wip {
  background: rgba(210, 236, 255, 0.9);
  color: #1d536f;
  border-color: rgba(148, 196, 238, 0.7);
}

.project-list__chip--completed {
  background: rgba(210, 244, 225, 0.92);
  color: #1e6042;
  border-color: rgba(140, 205, 170, 0.7);
}

.project-list__actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.project-list__detail-btn {
  border: 1px solid rgba(124, 205, 182, 0.25);
  border-radius: 12px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  background: linear-gradient(135deg, rgba(124, 205, 182, 0.14), rgba(146, 214, 222, 0.06));
  color: #114f45;
  box-shadow: none;
  transition:
    transform 0.18s ease,
    box-shadow 0.12s ease,
    background 0.12s ease;
}

.project-list__detail-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 18px rgba(124, 205, 182, 0.08);
  background: #eafaf6;
}

.project-list__loading,
.project-list__empty {
  padding: 40px 0;
  text-align: center;
  font-size: 14px;
  color: #4a5f7d;
}

@media (max-width: 1080px) {
  .project-list__item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .project-list__actions {
    width: 100%;
    justify-content: flex-end;
  }
}
</style>
