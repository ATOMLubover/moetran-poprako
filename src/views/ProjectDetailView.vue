<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import { Line } from 'vue-chartjs';
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Filler,
} from 'chart.js';
import { useToastStore } from '../stores/toast';
import { useUserStore } from '../stores/user';
import { useRouterStore } from '../stores/router';
import { getProjectTargets, getProjectFiles } from '../ipc/project';

// TODO: 后续接口可能扩展字段（如权限、进度来源），保持结构可扩展
interface PageMarkerData {
  // 单页标记统计（当前仅使用 source_count）
  pageNumber: number;
  markerCount: number;
}

// TODO: 状态枚举可能会增加（如 paused / archived）
type ProjectStatus = 'pending' | 'in_progress' | 'completed';

import type { ResMember } from '../api/model/member';
// TODO: 项目详情字段未来可能会增加 Poprako 相关统计 / 团队角色等
interface ProjectDetail {
  id: string;
  title: string;
  projsetName: string | null;
  projsetIndex: number | null;
  totalPages: number;
  totalMarkers: number;
  totalTranslatedMarkers: number;
  totalProofreadMarkers: number;
  translationStatus: ProjectStatus;
  proofreadingStatus: ProjectStatus;
  letteringStatus: ProjectStatus;
  reviewingStatus: ProjectStatus;
  translators: string[];
  proofreaders: string[];
  letterers: string[];
  reviewers: string[];
  pageMarkers: PageMarkerData[];
}

const props = defineProps<{
  projectId: string;
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
  // PopRaKo members with typed ResMember
  members?: ResMember[];
  // whether this project is backed by PopRaKo
  hasPoprako?: boolean;
  // principals are user ids
  principals?: string[];
  // publish state
  isPublished?: boolean;
  // Moetran 原生项目可能返回的 role 字段；若非 null 表示当前用户在该项目中
  role?: any | null;
}>();

interface FileInfo {
  id: string;
  name: string;
  sourceCount: number;
  url: string; // 图片URL（Moetran API 总是返回）
}

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'open-modifier'): void;
}>();

const toastStore = useToastStore();
const routerStore = useRouterStore();

// 项目详情数据
const project = ref<ProjectDetail | null>(null);

// 存储 target 和 files 列表供翻译视图使用
const primaryTargetId = ref<string | null>(null);
const primaryFiles = ref<FileInfo[]>([]);

// NOTE: 修改按钮现在将触发打开创建/修改面板（由父组件处理）

// 详细标记数据加载状态
const loadingMarkers = ref(false);
// 卡片容器引用（用于后续可能的动态高度微调）
const cardRef = ref<HTMLElement | null>(null);

// 是否为项目管理员（后续可根据真实用户身份判断）
// (已由 'principal' 角色控制关键编辑权限)

const userStore = useUserStore();
const isMeInProject = computed(() => {
  const uid = userStore.user?.id;
  if (!uid) return false;
  // For native Moetran projects (no PopRaKo backing), Moetran may include a
  // top-level `role` field indicating the current user's role in the project.
  // If `hasPoprako === false`, treat `props.role !== null` as membership.
  if (props.hasPoprako === false) {
    // props.role may be undefined if backend didn't provide it; treat undefined as not-in-project
    return (props.role ?? null) !== null;
  }

  // Prefer explicit PopRaKo `members` (each member exposes `userId`),
  // fall back to legacy role arrays which may contain user ids.
  if (props.members && Array.isArray(props.members) && props.members.length > 0) {
    return (props.members as ResMember[]).some(
      m => (m.userId ?? (m as unknown as { user_id?: string }).user_id ?? m.memberId) === uid
    );
  }

  const members = [
    ...(props.translators || []),
    ...(props.proofreaders || []),
    ...(props.letterers || []),
    ...(props.reviewers || []),
  ];
  return members.includes(uid);
});

// whether current user is the project's principal (if principals provided)
const isMePrincipal = computed(() => {
  const uid = userStore.user?.id;
  if (!uid) return false;
  const principals = props.principals ?? [];
  return principals.includes(uid);
});

// whether current user is translator or proofreader
const isMeTranslatorOrProofreader = computed(() => {
  const uid = userStore.user?.id;
  if (!uid) return false;

  // For native Moetran projects, if backend provided a `role` field,
  // treat any non-null value as membership (allowing 翻校 entry).
  if (props.hasPoprako === false) {
    return (props.role ?? null) !== null;
  }
  console.log('Checking isMeTranslatorOrProofreader for uid', uid, {
    members: props.members,
    translators: props.translators,
    proofreaders: props.proofreaders,
  });

  // If PopRaKo `members` provided with role flags, prefer that
  if (props.members && Array.isArray(props.members) && props.members.length > 0) {
    const m = (props.members as ResMember[]).find(
      mm => (mm.userId ?? (mm as unknown as { user_id?: string }).user_id ?? mm.memberId) === uid
    );
    return !!m && (m.isTranslator === true || m.isProofreader === true);
  }

  // Fallback to legacy role arrays
  const translators = props.translators ?? [];
  const proofreaders = props.proofreaders ?? [];
  return translators.includes(uid) || proofreaders.includes(uid);
});

// --- 进度相关计算 ---
// (per-page and totals are displayed as raw numbers; percent progress computations removed)

// 状态文本与配色（简约风格）
const statusText: Record<ProjectStatus, string> = {
  pending: '未开始',
  in_progress: '进行中',
  completed: '已完成',
};

const statusColorMap: Record<ProjectStatus, string> = {
  pending: '#95A8BC',
  in_progress: '#2F6FAE',
  completed: '#3A9E5E',
};

// 汇总四类工作状态
const statusBlocks = computed(() => {
  if (!project.value)
    return [] as Array<{ label: string; members: string[]; status: ProjectStatus }>;
  // If PopRaKo `members` prop is provided, prefer extracting roles from the members' flags.
  if (props.members && Array.isArray(props.members) && props.members.length > 0) {
    const members = props.members as ResMember[];

    const pickName = (m: ResMember) => m.username || m.userId || m.memberId;

    const translators = members.filter(m => m.isTranslator).map(pickName);
    const proofreaders = members.filter(m => m.isProofreader).map(pickName);
    const letterers = members.filter(m => m.isTypesetter).map(pickName);
    // Map principals to the "监修"/reviewers slot (PopRaKo exposes isPrincipal)
    const reviewers = members.filter(m => m.isPrincipal).map(pickName);

    return [
      { label: '翻译', members: translators, status: project.value.translationStatus },
      { label: '校对', members: proofreaders, status: project.value.proofreadingStatus },
      { label: '嵌字', members: letterers, status: project.value.letteringStatus },
      { label: '监修', members: reviewers, status: project.value.reviewingStatus },
    ];
  }

  // Fallback: use legacy arrays on project.value (strings)
  return [
    { label: '翻译', members: project.value.translators, status: project.value.translationStatus },
    {
      label: '校对',
      members: project.value.proofreaders,
      status: project.value.proofreadingStatus,
    },
    { label: '嵌字', members: project.value.letterers, status: project.value.letteringStatus },
    { label: '监修', members: project.value.reviewers, status: project.value.reviewingStatus },
  ];
});

// Chart.js 注册组件
ChartJS.register(
  Title,
  Tooltip,
  Legend,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Filler
);

// 视口高度与动态图表高度控制，避免出现垂直滚动条
const windowHeight = ref(window.innerHeight);
function handleResize() {
  windowHeight.value = window.innerHeight;
}

// 预留顶部/其他板块空间，动态计算图表高度（在给定范围内）
const chartDynamicHeight = computed(() => {
  const reserved = 620; // 估算其他内容占用高度
  const available = windowHeight.value - reserved;
  return Math.max(180, Math.min(300, available));
});

// Chart.js 数据
const chartData = computed(() => {
  if (!project.value || project.value.pageMarkers.length === 0) {
    return {
      labels: [],
      datasets: [],
    };
  }
  return {
    labels: project.value.pageMarkers.map(p => 'P' + p.pageNumber),
    datasets: [
      {
        label: '标记数',
        data: project.value.pageMarkers.map(p => p.markerCount),
        borderColor: '#3d79c4',
        backgroundColor: 'rgba(61,121,196,0.25)',
        tension: 0.28,
        pointRadius: 3.6,
        fill: true,
      },
    ],
  };
});

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: true, labels: { font: { size: 12 } } },
    title: { display: false },
    tooltip: { intersect: false },
  },
  scales: {
    x: {
      ticks: { font: { size: 11 } },
      grid: { color: 'rgba(140,180,220,0.15)' },
    },
    y: {
      beginAtZero: true,
      ticks: { font: { size: 11 } },
      grid: { color: 'rgba(140,180,220,0.15)' },
    },
  },
}));

// 数字状态映射到 ProjectStatus
function mapStatusNumberToProjectStatus(n: number | null): ProjectStatus {
  if (n === 2) return 'completed';
  if (n === 1) return 'in_progress';
  return 'pending';
}

// 加载项目详情与标记分布
async function loadProject(): Promise<void> {
  try {
    const base: ProjectDetail = {
      id: props.projectId,
      title: props.title,
      projsetName: props.projsetName,
      projsetIndex: props.projsetIndex,
      totalPages: 0,
      totalMarkers: props.totalMarkers ?? 0,
      totalTranslatedMarkers: props.totalTranslated ?? 0,
      totalProofreadMarkers: props.totalChecked ?? 0,
      translationStatus: mapStatusNumberToProjectStatus(props.translatingStatus),
      proofreadingStatus: mapStatusNumberToProjectStatus(props.proofreadingStatus),
      letteringStatus: mapStatusNumberToProjectStatus(props.typesettingStatus),
      reviewingStatus: mapStatusNumberToProjectStatus(props.reviewingStatus),
      translators: props.translators ?? [],
      proofreaders: props.proofreaders ?? [],
      letterers: props.letterers ?? [],
      reviewers: props.reviewers ?? [],
      pageMarkers: [],
    };

    project.value = base;

    // 拉取 targets 和 files，仅用于 totalPages 与每页 source_count
    loadingMarkers.value = true;

    console.debug('[ProjectDetail] loadProject start', { projectId: props.projectId, base });

    // Determine whether to fetch targets (only if current user is a member)
    let files;
    if (isMeInProject.value) {
      // targets
      let targets;
      try {
        targets = await getProjectTargets(props.projectId);
        console.debug('[ProjectDetail] getProjectTargets', { projectId: props.projectId, targets });
      } catch (e) {
        console.error('[ProjectDetail] getProjectTargets failed', {
          projectId: props.projectId,
          error: e,
        });
        loadingMarkers.value = false;
        toastStore.show('加载项目 targets 失败: ' + (e?.toString?.() ?? String(e)), 'error');
        return;
      }

      if (!targets || !targets.length) {
        console.debug('[ProjectDetail] no targets found', { projectId: props.projectId });
        loadingMarkers.value = false;
        return;
      }

      const primaryTarget = targets[0];

      // 存储 targetId 供翻译视图使用
      primaryTargetId.value = primaryTarget.id;

      // files (with target filter)
      try {
        files = await getProjectFiles(props.projectId, primaryTarget.id);
        console.debug('[ProjectDetail] getProjectFiles', {
          projectId: props.projectId,
          targetId: primaryTarget.id,
          files,
        });

        // If target-specific fetch returned no files, try fallback without target filter.
        if (!files || files.length === 0) {
          console.debug('[ProjectDetail] target-specific files empty, trying unfiltered files', {
            projectId: props.projectId,
          });
          try {
            const unfiltered = await getProjectFiles(props.projectId);
            console.debug('[ProjectDetail] getProjectFiles (fallback)', {
              projectId: props.projectId,
              unfiltered,
            });
            if (unfiltered && unfiltered.length > 0) files = unfiltered;
          } catch (e2) {
            // fallback failed; keep original empty list and continue to show no files
            console.warn('[ProjectDetail] fallback getProjectFiles failed', {
              projectId: props.projectId,
              error: e2,
            });
          }
        }
      } catch (e) {
        console.error('[ProjectDetail] getProjectFiles failed', {
          projectId: props.projectId,
          targetId: primaryTarget.id,
          error: e,
        });
        loadingMarkers.value = false;
        toastStore.show('加载项目 files 失败: ' + (e?.toString?.() ?? String(e)), 'error');
        return;
      }
    } else {
      // Not a member: skip targets fetch and request files without target filter
      try {
        console.debug('[ProjectDetail] user not member, fetching files without target', {
          projectId: props.projectId,
        });
        files = await getProjectFiles(props.projectId);
        console.debug('[ProjectDetail] getProjectFiles (no target)', {
          projectId: props.projectId,
          files,
        });
      } catch (e) {
        console.error('[ProjectDetail] getProjectFiles failed (no target)', {
          projectId: props.projectId,
          error: e,
        });
        loadingMarkers.value = false;
        toastStore.show('加载项目 files 失败: ' + (e?.toString?.() ?? String(e)), 'error');
        return;
      }
    }

    if (project.value) {
      project.value.totalPages = (files || []).length;
      project.value.pageMarkers = (files || []).map((f, index) => ({
        pageNumber: index + 1,
        markerCount: f.sourceCount,
      }));

      // 存储整个文件列表供翻译视图使用
      if (files && files.length > 0) {
        primaryFiles.value = files;
        // 如果还没有 targetId（非成员情况），也允许阅读模式
        // 非成员看不到 targets，但可以看文件，此时使用空字符串作为占位
        if (!primaryTargetId.value && !isMeInProject.value) {
          primaryTargetId.value = ''; // 允许非成员进入阅读模式
        }
      }
    }

    loadingMarkers.value = false;
  } catch (err) {
    loadingMarkers.value = false;
    toastStore.show('加载项目详情失败', 'error');
  }
}

// 切换修改面板显隐
function handleToggleModifier(): void {
  // 请求父组件打开修改面板
  emit('open-modifier');
}

// 加入或退出项目（后续需要真实接口）
function handleJoinOrLeave(): void {
  // 加入或退出项目
  if (!project.value) return;

  if (isMeInProject.value) {
    toastStore.show('已退出项目');
    // TODO: 真实退出动作
  } else {
    toastStore.show('已加入项目');
    // TODO: 真实加入动作
  }
}

// 进入翻译工作台（带翻校模式或阅读模式）
function openTranslator(enabled: boolean): void {
  // 验证必要数据是否已加载
  // 注意：非成员用户在阅读模式下，primaryTargetId 可以为空字符串
  if (primaryTargetId.value === null || primaryFiles.value.length === 0) {
    toastStore.show('项目数据尚未加载完成，请稍后再试', 'error');
    return;
  }

  // 使用 router store 导航到翻译视图（全屏独立视图）
  routerStore.navigateToTranslator({
    projectId: props.projectId,
    targetId: primaryTargetId.value,
    files: primaryFiles.value,
    enabled,
    initialMode: enabled ? 'translate' : 'read',
  });
}

// 关闭当前详情视图
function handleClose(): void {
  // 关闭详情
  emit('close');
}

onMounted(() => {
  loadProject();
  window.addEventListener('resize', handleResize);
});

// 如果父组件在侧栏打开时切换了 `projectId`，需要响应式地重新加载数据
watch(
  () => props.projectId,
  (newId, oldId) => {
    if (newId === oldId) return;
    project.value = null;
    primaryTargetId.value = null;
    primaryFiles.value = [];
    loadingMarkers.value = false;
    // 重新加载新的 project 数据
    loadProject();
  }
);

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
});
</script>

<template>
  <section class="project-detail" v-if="project">
    <header class="pd-header">
      <div class="pd-header__left">
        <h1 class="pd-title">{{ project.title }}</h1>
      </div>
      <div class="pd-header__right">
        <button type="button" class="pd-refresh" @click="loadProject" title="刷新">
          <svg
            width="20"
            height="20"
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
        <button type="button" class="pd-close" @click="handleClose" title="关闭">×</button>
      </div>
    </header>

    <div class="pd-topbar">
      <div class="pd-tags">
        <span class="pd-tag">项目集: {{ project.projsetName ?? '-' }}</span>
        <span class="pd-tag">项目序号: {{ project.projsetIndex ?? '-' }}</span>
      </div>
      <div class="pd-actions">
        <button
          v-if="isMePrincipal && props.hasPoprako !== false"
          type="button"
          class="pd-btn pd-btn--primary"
          @click="handleToggleModifier"
          :disabled="props.isPublished"
          :title="props.isPublished ? '已发布的项目无法修改' : '修改项目信息'"
        >
          修改项目信息
        </button>
        <button v-else type="button" class="pd-btn pd-btn--primary" @click="handleJoinOrLeave">
          {{ isMeInProject ? '退出项目' : '加入项目' }}
        </button>
        <button
          v-if="isMeTranslatorOrProofreader"
          type="button"
          class="pd-btn pd-btn--secondary"
          @click="openTranslator(true)"
        >
          翻校
        </button>
        <button type="button" class="pd-btn pd-btn--secondary" @click="openTranslator(false)">
          阅读
        </button>
      </div>
    </div>

    <div class="pd-card" ref="cardRef">
      <div class="pd-card__inner">
        <!-- <div class="pd-card__left">
          <CircularProgress
            :progress="translationProgress"
            color="yellow"
            label="翻译进度"
            class="pd-circ"
          />
          <CircularProgress
            :progress="proofreadProgress"
            color="pink"
            label="校对进度"
            class="pd-circ"
          />
        </div> -->
        <div class="pd-card__right">
          <h2 class="pd-block-title">项目概况</h2>
          <!-- 状态卡片栅格 -->
          <div class="pd-status-cards">
            <div
              v-for="blk in statusBlocks"
              :key="blk.label"
              class="pd-status-card"
              :style="{ borderColor: statusColorMap[blk.status] }"
            >
              <div class="pd-status-card__label">{{ blk.label }}</div>
              <div class="pd-status-card__state" :style="{ color: statusColorMap[blk.status] }">
                {{ statusText[blk.status] }}
              </div>
            </div>
          </div>
          <!-- 指标统计栅格 -->
          <div class="pd-metrics-grid">
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">总页数</span>
              <span class="pd-metric-box__value">{{ project.totalPages }}</span>
            </div>
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">总标记数</span>
              <span class="pd-metric-box__value">{{ project.totalMarkers }}</span>
            </div>
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">已翻译</span>
              <span class="pd-metric-box__value pd-metric-box__value--yellow">{{
                project.totalTranslatedMarkers
              }}</span>
            </div>
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">已校对</span>
              <span class="pd-metric-box__value pd-metric-box__value--pink">{{
                project.totalProofreadMarkers
              }}</span>
            </div>
          </div>
          <!-- 成员标签行 -->
          <div class="pd-members-tags">
            <div v-for="blk in statusBlocks" :key="blk.label + '-members'" class="pd-member-chip">
              <span class="pd-member-chip__role">[{{ blk.label }}]</span>
              <span class="pd-member-chip__list">
                <template v-if="blk.members && blk.members.length > 0">
                  {{ blk.members.join('、') }}
                </template>
                <template v-else>未分配</template>
              </span>
            </div>
          </div>
          <!-- 指标统计栅格 -->
          <!-- <div class="pd-metrics-grid">
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">总页数</span>
              <span class="pd-metric-box__value">{{ project.totalPages }}</span>
            </div>
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">总标记数</span>
              <span class="pd-metric-box__value">{{ project.totalMarkers }}</span>
            </div>
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">已翻译标记数</span>
              <span class="pd-metric-box__value pd-metric-box__value--yellow">{{
                project.totalTranslatedMarkers
              }}</span>
            </div>
            <div class="pd-metric-box">
              <span class="pd-metric-box__label">已校对标记数</span>
              <span class="pd-metric-box__value pd-metric-box__value--pink">{{
                project.totalProofreadMarkers
              }}</span>
            </div>
          </div> -->
        </div>
      </div>
    </div>

    <div class="pd-chart-block">
      <h2 class="pd-subtitle">每页标记分布</h2>
      <div
        class="pd-chart-canvas"
        :class="{ 'pd-chart-canvas--loading': loadingMarkers }"
        :style="{ height: chartDynamicHeight + 'px' }"
      >
        <Line v-if="chartData.labels.length > 0" :data="chartData" :options="chartOptions" />
        <div v-else class="pd-chart-empty">暂无标记数据</div>
      </div>
    </div>
  </section>
  <section v-else class="project-detail__loading">
    <div class="pd-loading">加载项目详情...</div>
  </section>
</template>

<style scoped>
/* 详情视图整体采用视口高度，并通过内部 flex 避免出现外部滚动条 */
.project-detail {
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  height: 100vh;
  padding: 16px 22px 18px;
  background: linear-gradient(180deg, #f5f9ff 0%, #ffffff 65%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
  gap: 10px;
}

.pd-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 28px;
}

.pd-header__left {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pd-title {
  margin: 0;
  font-size: 26px;
  font-weight: 600;
  letter-spacing: 0.5px;
}
.pd-meta {
  margin: 0;
  font-size: 13px;
  color: #4a5f7a;
}

.pd-header__right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.pd-refresh {
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  padding: 6px 8px;
  border-radius: 10px;
  cursor: pointer;
  transition:
    background 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.15s ease;
}
.pd-refresh:hover {
  background: #eef6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.25);
  transform: translateY(-1px);
}
.pd-close {
  border: 1px solid rgba(150, 180, 210, 0.5);
  background: #fff;
  color: #2a3d52;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  cursor: pointer;
  font-size: 20px;
  line-height: 34px;
  padding: 0;
  transition:
    box-shadow 0.2s ease,
    transform 0.2s ease;
}
.pd-close:hover {
  box-shadow: 0 8px 20px rgba(140, 180, 230, 0.25);
  transform: translateY(-2px);
}

.pd-topbar {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.pd-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.pd-tag {
  font-size: 12px;
  padding: 6px 14px;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.45);
  border-radius: 999px;
  color: #2d4a63;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.pd-tag--ok {
  background: #e8f8ef;
  border-color: rgba(80, 160, 120, 0.5);
  color: #2f6b4b;
}
.pd-tag--bad {
  background: #fff3f3;
  border-color: rgba(210, 120, 120, 0.5);
  color: #823c3c;
}
.pd-tag--warn {
  background: #f9f1ff;
  border-color: rgba(180, 120, 210, 0.5);
  color: #6a3d88;
}

.pd-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.pd-btn {
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.3px;
  border-radius: 999px;
  cursor: pointer;
  border: 1px solid;
  background: #fff;
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease;
}
.pd-btn--primary {
  border-color: rgba(118, 184, 255, 0.7);
  color: #2f5a8f;
}
.pd-btn--secondary {
  border-color: rgba(140, 180, 210, 0.55);
  color: #32526d;
}
.pd-btn:hover {
  box-shadow: 0 8px 24px rgba(136, 190, 247, 0.28);
  transform: translateY(-2px);
}
.pd-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

.pd-description {
  margin-top: 4px;
}
.pd-subtitle {
  margin: 0 0 10px;
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.4px;
  color: #23415b;
}
.pd-subtitle--inline {
  margin: 0 0 14px;
}
.pd-desc-text {
  margin: 0;
  font-size: 14px;
  line-height: 1.7;
  color: #445b72;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  padding: 14px 18px;
  border-radius: 12px;
}

.pd-progress {
  margin-top: 26px;
  display: flex;
  flex-wrap: wrap;
  gap: 26px;
  align-items: stretch;
}
.pd-progress__block {
  flex: 0 0 220px;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 14px;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: 0 6px 20px rgba(140, 180, 230, 0.18);
}
.pd-progress__value {
  font-size: 24px;
  font-weight: 600;
  color: #1f2e43;
}
.pd-progress__label {
  font-size: 12px;
  color: #5b768e;
}
.pd-progress__bar {
  position: relative;
  height: 10px;
  background: #ecf4fb;
  border-radius: 6px;
  overflow: hidden;
}
.pd-progress__bar-fill {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  background: linear-gradient(90deg, #66aaf7, #4991e4);
  border-radius: 6px;
}
.pd-progress__bar-fill--proof {
  background: linear-gradient(90deg, #ff9abb, #e86b95);
}

.pd-stats {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 16px;
}
.pd-stat {
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 14px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  box-shadow: 0 6px 20px rgba(140, 180, 230, 0.16);
}
.pd-stat__label {
  font-size: 12px;
  color: #5b768e;
}
.pd-stat__value {
  font-size: 22px;
  font-weight: 600;
  color: #23415b;
}
.pd-stat__value--yellow {
  color: #b98119;
}
.pd-stat__value--pink {
  color: #b24874;
}

.pd-status-grid {
  margin-top: 32px;
}
.pd-status__list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 18px;
}
.pd-status__item {
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 14px;
  padding: 14px 16px 16px;
  box-shadow: 0 6px 20px rgba(140, 180, 230, 0.16);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pd-status__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.pd-status__label {
  font-size: 13px;
  font-weight: 600;
  color: #2d4a63;
}
.pd-status__state {
  font-size: 11px;
  padding: 4px 10px;
  color: #fff;
  border-radius: 999px;
  letter-spacing: 0.5px;
}
.pd-status__members {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: #466079;
  min-height: 18px;
}

.pd-chart {
  margin-top: 34px;
}
.pd-chart__viewport {
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 14px;
  padding: 18px 22px;
  box-shadow: 0 6px 20px rgba(140, 180, 230, 0.16);
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 220px;
}
.pd-chart__viewport--loading {
  opacity: 0.6;
}
.pd-chart__svg {
  width: 100%;
  height: auto;
  display: block;
}
.pd-chart__empty {
  font-size: 13px;
  color: #6b859d;
}

/* === 新增：卡片与二维布局样式，修复重叠与无分割问题 === */
.pd-card {
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.45);
  border-radius: 18px;
  padding: 14px 18px;
  box-shadow: 0 10px 32px rgba(140, 180, 230, 0.18);
}
.pd-card__inner {
  display: flex;
  align-items: stretch;
  gap: 32px;
}
.pd-card__left {
  flex: 0 0 150px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 22px;
  padding: 0 0 4px;
  border-right: 1px solid rgba(150, 180, 210, 0.35);
  box-sizing: border-box;
}
.pd-card__right {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
}
.pd-block-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  padding-bottom: 10px;
  border-bottom: 1px solid rgba(150, 180, 210, 0.35);
  color: #23415b;
}
.pd-status-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
  gap: 10px;
}
.pd-status-card {
  background: #f8fbff;
  border: 2px solid;
  border-radius: 14px;
  padding: 10px 12px 12px;
  text-align: center;
  box-shadow: 0 6px 18px rgba(140, 180, 230, 0.16);
}
.pd-status-card__label {
  font-size: 13px;
  font-weight: 600;
  color: #2d4a63;
  margin-bottom: 4px;
}
.pd-status-card__state {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.6px;
}
.pd-members-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.pd-member-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 999px;
  padding: 6px 14px 6px 16px;
  font-size: 12px;
  box-shadow: 0 4px 14px rgba(140, 180, 230, 0.14);
}
.pd-member-chip__role {
  font-weight: 700;
  color: #2d4a63;
}
.pd-member-chip__list {
  font-weight: 500;
  color: #466079;
}
.pd-metrics-grid {
  display: flex;
  flex-direction: row;
  align-items: stretch;
  gap: 10px;
}
.pd-metric-box {
  flex: 1 1 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #eef5fc;
  border: 2px solid #d5e3f1;
  padding: 12px 16px;
  border-radius: 14px;
  font-size: 13px;
  font-weight: 500;
  color: #345067;
  transition: background 0.25s ease;
}
.pd-metric-box:hover {
  background: #e4f0fa;
}
.pd-metric-box__value {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: #23415b;
}
.pd-metric-box__value--yellow {
  color: #b98119;
}
.pd-metric-box__value--pink {
  color: #b24874;
}

/* Chart block adjustments */
.pd-chart-block {
  margin-top: 8px;
}
.pd-chart-canvas {
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.45);
  border-radius: 18px;
  padding: 16px 20px;
  box-shadow: 0 12px 36px rgba(140, 180, 230, 0.18);
  box-sizing: border-box;
  position: relative;
}
.pd-chart-canvas--loading {
  opacity: 0.65;
}
.pd-chart-empty {
  font-size: 13px;
  color: #6b859d;
}

@media (max-width: 1100px) {
  .pd-card__inner {
    flex-direction: column;
  }
  .pd-card__left {
    flex: none;
    width: 100%;
    flex-direction: row;
    justify-content: center;
    border-right: none;
    border-bottom: 1px solid rgba(150, 180, 210, 0.35);
    padding-bottom: 18px;
    gap: 46px;
  }
  .pd-card__right {
    padding-top: 4px;
  }
}

.project-detail__loading {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  background: linear-gradient(180deg, #f5f9ff 0%, #ffffff 65%);
}
.pd-loading {
  font-size: 16px;
  color: #4a5f7a;
}

@media (max-width: 960px) {
  .pd-progress {
    flex-direction: column;
  }
  .pd-progress__block {
    width: 100%;
  }
  .pd-stats {
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  }
  .pd-status__list {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
  .project-detail {
    padding: 16px 16px 18px;
  }
}
</style>
