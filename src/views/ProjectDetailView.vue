<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
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
import { useCacheStore } from '../stores/cache';
import { getProjectTargets, getProjectFiles, proxyImage, updateProjStatus } from '../ipc/project';
import LoadingCircle from '../components/LoadingCircle.vue';
import FileUploader from '../components/FileUploader.vue';
import {
  checkFileCache,
  downloadProjectFiles,
  deleteFileCache,
  type FileDownloadInfo,
} from '../ipc/image_cache';

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
  translationStatusValue: number;
  proofreadingStatus: ProjectStatus;
  proofreadingStatusValue: number;
  letteringStatus: ProjectStatus;
  letteringStatusValue: number;
  reviewingStatus: ProjectStatus;
  reviewingStatusValue: number;
  translators: string[];
  proofreaders: string[];
  letterers: string[];
  reviewers: string[];
  pageMarkers: PageMarkerData[];
}

type StatusType = 'translating' | 'proofreading' | 'typesetting' | 'reviewing';

const statusNameMap: Record<number, string> = {
  0: '未开始',
  1: '进行中',
  2: '已完成',
};

const statusOperationLabelMap: Record<StatusType, string> = {
  translating: '翻译',
  proofreading: '校对',
  typesetting: '嵌字',
  reviewing: '监修',
};

// 私有类型：Moetran 原生项目中可能带回的 role 结构，前端仅用作存在性检查
interface _ProjectRole {
  [key: string]: unknown;
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
  // team id for member management
  teamId?: string;
  // team name for display
  teamName?: string;
  // Moetran 原生项目可能返回的 role 字段；若非 null 表示当前用户在该项目中
  role?: _ProjectRole | null;
}>();

interface FileInfo {
  id: string;
  name: string;
  sourceCount: number;
  url: string; // 图片URL（Moetran API 总是返回）
  coverUrl?: string; // 低分辨率预览图（可选）
}

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'open-modifier'): void;
}>();

const toastStore = useToastStore();
const routerStore = useRouterStore();
const cacheStore = useCacheStore();

// 项目详情数据
const project = ref<ProjectDetail | null>(null);

// 存储 target 和 files 列表供翻译视图使用
const primaryTargetId = ref<string | null>(null);
const primaryFiles = ref<FileInfo[]>([]);

// NOTE: 修改按钮现在将触发打开创建/修改面板（由父组件处理）

// 详细标记数据加载状态
const loadingMarkers = ref(false);
const loadingMarkersFailed = ref(false);

// 图片缓存相关状态
const hasCachedFiles = ref(false);
const isDownloading = ref(false);
const isDeleting = ref(false);

// 文件上传对话框状态
const showUploader = ref(false);

// 封面图片加载状态
const coverImageData = ref<string | null>(null);
const loadingCover = ref(false);

// 预览分页状态（使用 coverUrl，不走缓存）
const previewPage = ref(1);
// 不再使用固定值，而是根据预览网格容器宽度与每个预览项的最大宽度动态计算每页能放多少个（单行展示）
const previewGridRef = ref<HTMLElement | null>(null);
const previewGridWidth = ref(0);
// 控制是否处于测量模式：测量时渲染全部项（不触发图片加载），测量后切换为按页渲染
const measuringPreviews = ref(false);
// 可写的每页数量，由测量或 resize 事件设置
const previewsPerPageRef = ref(6);
const PREVIEW_ITEM_MIN = 140; // 与 CSS 中 min-width 保持一致
const PREVIEW_ITEM_MAX = 220; // 单个预览项的最大宽度（可调整）
const PREVIEW_GAP = 10; // grid gap（px），与 CSS 保持同步

// 计算单行最多能放多少预览项：使用 (width + gap) / (itemMax + gap) 的近似
const previewsPerPage = computed(() => previewsPerPageRef.value);

const totalPreviewPages = computed(() => {
  const p = previewsPerPage.value || 1;
  return Math.max(1, Math.ceil((primaryFiles.value || []).length / p));
});

const previewFiles = computed(() => {
  const per = previewsPerPage.value || 1;
  const start = (previewPage.value - 1) * per;
  return (primaryFiles.value || []).slice(start, start + per);
});

// 渲染用文件列表：测量模式下渲染全部 primaryFiles（不触发 proxy 请求），否则渲染按页切片
const renderPreviewFiles = computed(() => {
  return measuringPreviews.value ? primaryFiles.value : previewFiles.value;
});

// 生成用于绑定到预览 grid 的内联样式（控制列数，使其只显示单行）
const previewGridStyle = computed(() => {
  if (measuringPreviews.value) return {} as Record<string, string>;
  const n = previewsPerPage.value || 1;
  return {
    gridTemplateColumns: `repeat(${n}, minmax(${PREVIEW_ITEM_MIN}px, ${PREVIEW_ITEM_MAX}px))`,
  } as Record<string, string>;
});

// 当每页数量改变时，调整当前页码并重载当前页图片
watch(
  () => previewsPerPage.value,
  (newVal, oldVal) => {
    if (newVal === oldVal) return;
    previewPage.value = Math.min(previewPage.value, totalPreviewPages.value);
    loadPreviewPageImages().catch(err => console.warn('preview load failed', err));
  }
);

// preview images proxied via backend (object URLs stored here)
const previewImageMap = ref<Record<string, string | undefined>>({});
const loadingPreview = ref(false);

function revokePreviewUrlsExcept(keepIds: string[]) {
  const map = previewImageMap.value;
  for (const id of Object.keys(map)) {
    if (!keepIds.includes(id)) {
      // do not revoke object URLs immediately — DOM may still reference them
      delete map[id];
    }
  }
}

function cleanupAllPreviewUrls() {
  try {
    for (const v of Object.values(previewImageMap.value)) {
      if (v) {
        try {
          URL.revokeObjectURL(v);
        } catch (e) {
          // ignore
        }
      }
    }
  } finally {
    previewImageMap.value = {};
  }
}

async function loadPreviewPageImages() {
  const files = previewFiles.value || [];
  if (!files.length) {
    // clear existing previews
    revokePreviewUrlsExcept([]);
    previewImageMap.value = {};
    return;
  }

  loadingPreview.value = true;

  const keepIds: string[] = files.map(f => f.id);

  // revoke urls not needed anymore
  revokePreviewUrlsExcept(keepIds);

  for (const f of files) {
    if (previewImageMap.value[f.id]) continue; // already loaded
    try {
      const src = f.coverUrl ?? undefined;
      if (!src) {
        previewImageMap.value[f.id] = undefined;
        continue;
      }
      const res = await proxyImage(src);
      // store as data URL to avoid creating blob object URLs which may be revoked elsewhere
      previewImageMap.value[f.id] = `data:${res.content_type};base64,${res.b64}`;
    } catch (e) {
      console.error('[ProjectDetail] loadPreviewPageImages proxy failed', { id: f.id, e });
      previewImageMap.value[f.id] = undefined;
    }
  }

  loadingPreview.value = false;
}

function goPrevPreview() {
  if (previewPage.value > 1) previewPage.value -= 1;
}

function goNextPreview() {
  if (previewPage.value < totalPreviewPages.value) previewPage.value += 1;
}

// 是否为项目管理员（后续可根据真实用户身份判断）
// (已由 'principal' 角色控制关键编辑权限)

const userStore = useUserStore();
const currentUserId = computed(() => userStore.user?.id ?? '');

function extractMemberIdentity(member: ResMember): string {
  const legacy = (member as unknown as { user_id?: string }).user_id;
  return member.userId ?? legacy ?? member.memberId;
}

const currentMember = computed<ResMember | null>(() => {
  const uid = currentUserId.value;
  if (!uid || !props.members || props.members.length === 0) return null;
  return (props.members as ResMember[]).find(m => extractMemberIdentity(m) === uid) ?? null;
});

const isMeInProject = computed(() => {
  const uid = currentUserId.value;
  if (!uid) return false;
  if (props.hasPoprako === false) {
    return (props.role ?? null) !== null;
  }

  if (currentMember.value) return true;

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
const isMeTranslator = computed(() => {
  const uid = currentUserId.value;
  if (!uid) return false;
  if (props.members && props.members.length > 0) {
    return currentMember.value?.isTranslator === true;
  }
  return (props.translators ?? []).includes(uid);
});

const isMeProofreader = computed(() => {
  const uid = currentUserId.value;
  if (!uid) return false;
  if (props.hasPoprako === false) return false;
  if (props.members && props.members.length > 0) {
    return currentMember.value?.isProofreader === true;
  }
  return (props.proofreaders ?? []).includes(uid);
});

const isMeTypesetter = computed(() => {
  const uid = currentUserId.value;
  if (!uid) return false;
  if (props.members && props.members.length > 0) {
    return currentMember.value?.isTypesetter === true;
  }
  return (props.letterers ?? []).includes(uid);
});

const isMeReviewerRole = computed(() => {
  const uid = currentUserId.value;
  if (!uid) return false;
  if (props.members && props.members.length > 0) {
    return currentMember.value?.isPrincipal === true;
  }
  const reviewers = props.reviewers ?? [];
  if (reviewers.includes(uid)) return true;
  const principals = props.principals ?? [];
  return principals.includes(uid);
});

const isMeTranslatorOrProofreader = computed(() => {
  const uid = currentUserId.value;
  if (!uid) return false;
  if (props.hasPoprako === false) {
    return (props.role ?? null) !== null;
  }
  return isMeTranslator.value || isMeProofreader.value;
});

// whether current user is specifically a proofreader
// (保留命名供其他逻辑使用)

// --- 进度相关计算 ---
// (per-page and totals are displayed as raw numbers; percent progress computations removed)

// 状态配色（简约风格）
const statusColorMap: Record<ProjectStatus, string> = {
  pending: '#95A8BC',
  in_progress: '#2F6FAE',
  completed: '#3A9E5E',
};

function normalizeStatusValue(value: number | null | undefined): number {
  if (value === 1 || value === 2) return value;
  return 0;
}

function getStatusName(value: number): string {
  const normalized = value === 1 || value === 2 ? value : 0;
  return statusNameMap[normalized];
}

function defaultStatusUpdating(): Record<StatusType, boolean> {
  return {
    translating: false,
    proofreading: false,
    typesetting: false,
    reviewing: false,
  };
}

interface StatusConfirmState {
  visible: boolean;
  label: string;
  statusType: StatusType | null;
  nextStatus: number | null;
}

function createEmptyStatusConfirmState(): StatusConfirmState {
  return {
    visible: false,
    label: '',
    statusType: null,
    nextStatus: null,
  };
}

const statusUpdating = ref<Record<StatusType, boolean>>(defaultStatusUpdating());
const statusConfirm = ref<StatusConfirmState>(createEmptyStatusConfirmState());

const statusConfirmTargetName = computed(() => {
  const next = statusConfirm.value.nextStatus;
  if (typeof next !== 'number') return '';
  return getStatusName(next);
});

function resetStatusUpdating(): void {
  statusUpdating.value = defaultStatusUpdating();
}

function resetStatusConfirm(): void {
  statusConfirm.value = createEmptyStatusConfirmState();
}

function hasStatusPermission(type: StatusType): boolean {
  if (props.hasPoprako !== true) return false;
  switch (type) {
    case 'translating':
      return isMeTranslator.value;
    case 'proofreading':
      return isMeProofreader.value;
    case 'typesetting':
      return isMeTypesetter.value;
    case 'reviewing':
      return isMeReviewerRole.value;
    default:
      return false;
  }
}

function getNextStatusValue(current: number): number | null {
  if (current >= 2) return null;
  return current + 1;
}

function applyStatusValue(type: StatusType, value: number): void {
  if (!project.value) return;
  const mapped = mapStatusNumberToProjectStatus(value);
  switch (type) {
    case 'translating':
      project.value.translationStatusValue = value;
      project.value.translationStatus = mapped;
      break;
    case 'proofreading':
      project.value.proofreadingStatusValue = value;
      project.value.proofreadingStatus = mapped;
      break;
    case 'typesetting':
      project.value.letteringStatusValue = value;
      project.value.letteringStatus = mapped;
      break;
    case 'reviewing':
      project.value.reviewingStatusValue = value;
      project.value.reviewingStatus = mapped;
      break;
    default:
      break;
  }
}

// 汇总四类工作状态
// 私有类型：状态块
interface _StatusBlock {
  label: string;
  members: string[];
  status: ProjectStatus;
  statusValue: number;
  statusType: StatusType;
  stateText: string;
  canAdvance: boolean;
  isDisabled: boolean;
  isBusy: boolean;
}

const statusBlocks = computed(() => {
  if (!project.value) return [] as _StatusBlock[];
  const blocks: _StatusBlock[] = [];

  const pushBlock = (label: string, members: string[], statusType: StatusType): void => {
    if (!project.value) return;
    const valueMap: Record<StatusType, { status: ProjectStatus; value: number }> = {
      translating: {
        status: project.value.translationStatus,
        value: project.value.translationStatusValue,
      },
      proofreading: {
        status: project.value.proofreadingStatus,
        value: project.value.proofreadingStatusValue,
      },
      typesetting: {
        status: project.value.letteringStatus,
        value: project.value.letteringStatusValue,
      },
      reviewing: {
        status: project.value.reviewingStatus,
        value: project.value.reviewingStatusValue,
      },
    };
    const current = valueMap[statusType];
    const disabled = current.value >= 2;
    const busy = statusUpdating.value[statusType] === true;
    blocks.push({
      label,
      members,
      status: current.status,
      statusType,
      statusValue: current.value,
      stateText: getStatusName(current.value),
      canAdvance: hasStatusPermission(statusType) && !disabled && !contentLoading.value && !busy,
      isDisabled: disabled,
      isBusy: busy,
    });
  };

  if (props.members && Array.isArray(props.members) && props.members.length > 0) {
    const members = props.members as ResMember[];
    const pickName = (m: ResMember) => m.username || extractMemberIdentity(m);

    pushBlock('翻译', members.filter(m => m.isTranslator).map(pickName), 'translating');
    pushBlock('校对', members.filter(m => m.isProofreader).map(pickName), 'proofreading');
    pushBlock('嵌字', members.filter(m => m.isTypesetter).map(pickName), 'typesetting');
    pushBlock('监修', members.filter(m => m.isPrincipal).map(pickName), 'reviewing');
    return blocks;
  }

  pushBlock('翻译', project.value.translators, 'translating');
  pushBlock('校对', project.value.proofreaders, 'proofreading');
  pushBlock('嵌字', project.value.letterers, 'typesetting');
  pushBlock('监修', project.value.reviewers, 'reviewing');
  return blocks;
});

const confirmDialogBusy = computed(() => {
  const type = statusConfirm.value.statusType;
  if (!type) return false;
  return statusUpdating.value[type] === true;
});

function handleStatusCardClick(block: _StatusBlock): void {
  if (!block || !block.canAdvance || block.isDisabled || block.isBusy) return;
  const next = getNextStatusValue(block.statusValue);
  if (next === null) return;
  statusConfirm.value = {
    visible: true,
    label: block.label,
    statusType: block.statusType,
    nextStatus: next,
  };
}

function closeStatusConfirm(): void {
  resetStatusConfirm();
}

async function confirmStatusAdvance(): Promise<void> {
  const state = statusConfirm.value;
  if (!state.visible || state.statusType === null || state.nextStatus === null) {
    resetStatusConfirm();
    return;
  }
  const statusType = state.statusType;
  if (statusUpdating.value[statusType]) return;

  statusUpdating.value[statusType] = true;
  try {
    await updateProjStatus({
      projId: props.projectId,
      statusType,
      newStatus: state.nextStatus,
    });
    applyStatusValue(statusType, state.nextStatus);
    const roleLabel = statusOperationLabelMap[statusType];
    toastStore.show(`${roleLabel}状态已更新为${getStatusName(state.nextStatus)}`);
  } catch (error) {
    console.error('[ProjectDetail] updateProjStatus failed', {
      projectId: props.projectId,
      statusType,
      nextStatus: state.nextStatus,
      error,
    });
    toastStore.show('更新项目状态失败', 'error');
  } finally {
    statusUpdating.value[statusType] = false;
    resetStatusConfirm();
  }
}

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
  const reserved = 550; // 估算其他内容占用高度
  const available = windowHeight.value - reserved;
  return Math.max(200, Math.min(240, available));
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

// whether project content is still loading (used to disable actions)
const contentLoading = computed(() => {
  return loadingMarkers.value || project.value === null;
});

// 数字状态映射到 ProjectStatus
function mapStatusNumberToProjectStatus(n: number | null): ProjectStatus {
  if (n === 2) return 'completed';
  if (n === 1) return 'in_progress';
  return 'pending';
}

// 检查项目的图片缓存是否存在
async function checkCache(): Promise<void> {
  try {
    hasCachedFiles.value = await checkFileCache(props.projectId);
  } catch (err) {
    console.error('[ProjectDetail] checkFileCache failed', err);
    hasCachedFiles.value = false;
  }
}

// 下载项目的所有图片到本地缓存
async function handleDownloadCache(): Promise<void> {
  if (isDownloading.value || !primaryFiles.value.length) return;

  isDownloading.value = true;

  try {
    const files: FileDownloadInfo[] = primaryFiles.value.map(f => ({ url: f.url }));

    // 异步调用，不阻塞 UI
    downloadProjectFiles(props.projectId, props.title, files)
      .then(() => {
        toastStore.show('图片缓存下载完成');
        hasCachedFiles.value = true;
      })
      .catch(err => {
        console.error('图片缓存下载失败', err);
        toastStore.show('图片缓存下载失败', 'error');
      })
      .finally(() => {
        isDownloading.value = false;
      });

    toastStore.show('开始下载图片缓存...');
  } catch (err) {
    isDownloading.value = false;
    toastStore.show('启动下载失败', 'error');
  }
}

// 删除项目的图片缓存
async function handleDeleteCache(): Promise<void> {
  if (isDeleting.value) return;

  isDeleting.value = true;

  try {
    await deleteFileCache(props.projectId);
    hasCachedFiles.value = false;
    toastStore.show('图片缓存已删除');
  } catch (err) {
    toastStore.show('删除图片缓存失败', 'error');
  } finally {
    isDeleting.value = false;
  }
}

// 处理上传完成事件
function handleUploadComplete(successCount: number, failedCount: number): void {
  if (failedCount === 0) {
    toastStore.show(`成功上传 ${successCount} 个文件`);
  } else {
    toastStore.show(`上传完成：成功 ${successCount}，失败 ${failedCount}`, 'error');
  }
  // 可选：刷新项目文件列表
  loadProject();
}

// 加载封面图片（files 数组的第一个）
async function loadCoverImage(): Promise<void> {
  if (!primaryFiles.value.length || loadingCover.value) return;

  loadingCover.value = true;
  try {
    const firstFile = primaryFiles.value[0];

    // Try to reuse cached object URL first (key format used elsewhere)
    const key = `${props.projectId}::${firstFile.id}`;
    const cached = cacheStore.promoteImageCacheEntry(key);
    if (cached) {
      coverImageData.value = cached;
      return;
    }

    const result = await proxyImage(firstFile.url);

    // Use data URL directly to avoid object URL revoke races (no pinia cache)
    coverImageData.value = `data:${result.content_type};base64,${result.b64}`;
  } catch (err) {
    console.error('[ProjectDetail] loadCoverImage failed', err);
    coverImageData.value = null;
  } finally {
    loadingCover.value = false;
  }
}

// 加载项目详情与标记分布
async function loadProject(): Promise<void> {
  try {
    const translationValue = normalizeStatusValue(props.translatingStatus);
    const proofreadingValue = normalizeStatusValue(props.proofreadingStatus);
    const letteringValue = normalizeStatusValue(props.typesettingStatus);
    const reviewingValue = normalizeStatusValue(props.reviewingStatus);

    const base: ProjectDetail = {
      id: props.projectId,
      title: props.title,
      projsetName: props.projsetName,
      projsetIndex: props.projsetIndex,
      totalPages: 0,
      totalMarkers: props.totalMarkers ?? 0,
      totalTranslatedMarkers: props.totalTranslated ?? 0,
      totalProofreadMarkers: props.totalChecked ?? 0,
      translationStatus: mapStatusNumberToProjectStatus(translationValue),
      translationStatusValue: translationValue,
      proofreadingStatus: mapStatusNumberToProjectStatus(proofreadingValue),
      proofreadingStatusValue: proofreadingValue,
      letteringStatus: mapStatusNumberToProjectStatus(letteringValue),
      letteringStatusValue: letteringValue,
      reviewingStatus: mapStatusNumberToProjectStatus(reviewingValue),
      reviewingStatusValue: reviewingValue,
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

    let files;

    // Determine whether to fetch targets (only if current user is a member)
    if (isMeInProject.value) {
      // targets
      let targets;
      try {
        targets = await getProjectTargets(props.projectId);
        console.debug('[ProjectDetail] getProjectTargets', {
          projectId: props.projectId,
          targets,
        });
      } catch (e) {
        console.error('[ProjectDetail] getProjectTargets failed', {
          projectId: props.projectId,
          error: e,
        });
        loadingMarkers.value = false;
        loadingMarkersFailed.value = true;
        toastStore.show('加载项目 targets 失败', 'error');
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
        loadingMarkersFailed.value = true;
        toastStore.show('加载项目 files 失败', 'error');
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
        loadingMarkersFailed.value = true;
        toastStore.show('加载项目 files 失败', 'error');
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
        // 加载封面图片
        loadCoverImage();
        // 加载当前预览页的低分辨率图片（通过后端 proxy）
        previewPage.value = 1;
        loadPreviewPageImages().catch(err => console.warn('preview load failed', err));
      }
    }

    loadingMarkers.value = false;
  } catch (err) {
    loadingMarkers.value = false;
    loadingMarkersFailed.value = true;
    toastStore.show('加载项目详情失败', 'error');
  }
}

// 切换修改面板显隐
function handleToggleModifier(): void {
  // 请求父组件打开修改面板
  emit('open-modifier');
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
    hasPoprako: props.hasPoprako,
    isProofreader: isMeProofreader.value,
  });
}

// 关闭当前详情视图
function handleClose(): void {
  // 关闭详情
  emit('close');
}

onMounted(() => {
  loadProject();
  checkCache();
  window.addEventListener('resize', handleResize);
  // 初始化预览网格宽度，并使用 ResizeObserver 监听宽度变化以重新计算每页展示数量
  if (previewGridRef.value) {
    try {
      previewGridWidth.value = previewGridRef.value.clientWidth;
    } catch (e) {
      previewGridWidth.value = 0;
    }
  }
  const ro = new ResizeObserver(entries => {
    if (!entries || entries.length === 0) return;
    const r = entries[0];
    previewGridWidth.value = Math.floor(r.contentRect.width);
    // on resize, if not currently measuring, recompute per based on rendered item width
    if (!measuringPreviews.value) {
      recomputePerFromRenderedItems();
    }
  });
  // attach observer after a tick (element should be present)
  setTimeout(() => {
    if (previewGridRef.value) ro.observe(previewGridRef.value);
  }, 0);
  // store observer on ref for cleanup
  (previewGridRef as any).__ro = ro;
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
    loadingMarkersFailed.value = false;
    hasCachedFiles.value = false;
    resetStatusUpdating();
    resetStatusConfirm();
    // 重新加载新的 project 数据
    loadProject();
    checkCache();
    // reset preview cache when project changes
    cleanupAllPreviewUrls();
  }
);

// Load preview images whenever the preview page changes
watch(
  () => previewPage.value,
  () => {
    loadPreviewPageImages().catch(err => console.warn('preview load failed', err));
  }
);

// When primaryFiles change, reset preview page and reload
watch(
  () => primaryFiles.value.length,
  len => {
    if (len === 0) {
      previewImageMap.value = {};
      return;
    }
    // 进入测量模式：先渲染全部占位项以测量实际可用项宽度
    measuringPreviews.value = true;
    previewPage.value = 1;
    // 等待 DOM 更新后测量并切换回分页渲染
    nextTick(() => {
      // 使用 requestAnimationFrame 以确保布局完成
      requestAnimationFrame(() => {
        recomputePerFromRenderedItems();
      });
    });
  }
);

// 根据渲染出的子项宽度和容器宽度计算每页数量，并退出测量模式
function recomputePerFromRenderedItems(): void {
  try {
    const grid = previewGridRef.value;
    if (!grid) {
      previewsPerPageRef.value = 1;
      measuringPreviews.value = false;
      // load current page images (safe)
      loadPreviewPageImages().catch(err => console.warn('preview load failed', err));
      return;
    }

    const gridWidth = grid.clientWidth || previewGridWidth.value || 0;
    // 使用预设的最大项宽度计算每行数量，避免受 auto-fill 布局扩展的影响
    const per = Math.max(
      1,
      Math.floor((gridWidth + PREVIEW_GAP) / (PREVIEW_ITEM_MAX + PREVIEW_GAP))
    );
    previewsPerPageRef.value = per;

    // ensure current page is within new total pages
    previewPage.value = Math.min(
      previewPage.value,
      Math.max(1, Math.ceil(primaryFiles.value.length / per))
    );

    // exit measuring mode and load images for the computed page
    measuringPreviews.value = false;
    loadPreviewPageImages().catch(err => console.warn('preview load failed', err));
  } catch (e) {
    measuringPreviews.value = false;
    previewsPerPageRef.value = 1;
    loadPreviewPageImages().catch(err => console.warn('preview load failed', err));
  }
}

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
  // revoke preview object URLs and clear map
  cleanupAllPreviewUrls();
  // revoke cover image object URL
  if (coverImageData.value) {
    try {
      URL.revokeObjectURL(coverImageData.value);
    } catch (e) {
      // ignore
    }
  }
  // disconnect ResizeObserver if set
  try {
    const ro = (previewGridRef as any).__ro;
    if (ro && typeof ro.disconnect === 'function') ro.disconnect();
  } catch (e) {
    // ignore
  }
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
        <span v-if="props.teamName" class="pd-tag pd-tag--team">汉化组: {{ props.teamName }}</span>
        <span class="pd-tag">项目集: {{ project.projsetName ?? '-' }}</span>
        <span class="pd-tag">项目序号: {{ project.projsetIndex ?? '-' }}</span>
      </div>
      <div class="pd-actions">
        <button
          v-if="isMePrincipal && props.hasPoprako !== false && props.teamId"
          type="button"
          class="pd-btn pd-btn--primary"
          @click="handleToggleModifier"
          :disabled="props.isPublished || contentLoading"
          :title="
            props.isPublished
              ? '已发布的项目无法修改'
              : contentLoading
                ? '项目正在加载中...'
                : '修改项目信息'
          "
        >
          修改项目信息
        </button>
        <button
          v-else-if="isMePrincipal && props.hasPoprako !== false && !props.teamId"
          type="button"
          class="pd-btn pd-btn--primary"
          :disabled="true || contentLoading"
          :title="contentLoading ? '项目正在加载中...' : '缺少团队信息，无法修改项目'"
        >
          修改项目信息
        </button>
        <!-- join/leave moved to cover column to align with cover width -->
        <button
          v-if="isMeTranslatorOrProofreader"
          type="button"
          class="pd-btn pd-btn--secondary"
          @click="openTranslator(true)"
          :disabled="contentLoading"
          :title="contentLoading ? '项目正在加载中，稍后再试' : '进入翻校模式'"
        >
          翻校
        </button>
        <button
          type="button"
          class="pd-btn pd-btn--secondary"
          @click="openTranslator(false)"
          :disabled="contentLoading"
          :title="contentLoading ? '项目正在加载中，稍后再试' : '进入阅读模式'"
        >
          阅读
        </button>
        <button
          v-if="!hasCachedFiles"
          type="button"
          class="pd-btn pd-btn--secondary"
          @click="handleDownloadCache"
          :disabled="contentLoading || isDownloading || primaryFiles.length === 0"
          :title="
            contentLoading
              ? '项目正在加载中，稍后再试'
              : isDownloading
                ? '正在下载图片缓存'
                : '缓存图片到本地'
          "
        >
          {{ isDownloading ? '下载中...' : '缓存图片' }}
        </button>
        <button
          v-if="hasCachedFiles"
          type="button"
          class="pd-btn pd-btn--danger"
          @click="handleDeleteCache"
          :disabled="isDeleting"
        >
          {{ isDeleting ? '删除中...' : '删除缓存' }}
        </button>
        <button
          v-if="isMeInProject"
          type="button"
          class="pd-btn pd-btn--secondary"
          @click="showUploader = true"
          :disabled="contentLoading"
          :title="contentLoading ? '项目正在加载中，稍后再试' : '上传图片'"
        >
          上传图片
        </button>
      </div>
    </div>

    <FileUploader
      :visible="showUploader"
      :project-id="props.projectId"
      :project-name="props.title"
      @close="showUploader = false"
      @upload-complete="handleUploadComplete"
    />

    <div class="pd-main-layout">
      <!-- 左列：封面图片 -->
      <div class="pd-cover-column">
        <div class="pd-card__cover">
          <div v-if="loadingCover" class="pd-cover-loading">
            <LoadingCircle />
          </div>
          <div v-else-if="loadingMarkers || measuringPreviews" class="pd-cover-loading">
            <LoadingCircle />
          </div>
          <img
            v-else-if="coverImageData"
            :src="coverImageData"
            alt="项目封面"
            class="pd-cover-image"
          />
          <div v-else class="pd-cover-placeholder">暂无封面</div>
        </div>
        <!-- <div class="pd-cover-actions">
          <button
            v-if="showJoinLeave"
            type="button"
            :class="[
              'pd-btn',
              isMeInProject ? 'pd-btn--danger' : 'pd-btn--primary',
              'pd-cover-action-button',
            ]"
            @click="handleJoinOrLeave"
          >
            {{ isMeInProject ? '退出项目' : '加入项目' }}
          </button>
        </div> -->
      </div>

      <!-- 右列：所有内容 -->
      <div class="pd-content-column">
        <!-- 项目概况 -->
        <div class="pd-overview-card">
          <h2 class="pd-section-title">项目概况</h2>
          <div class="pd-status-cards">
            <div
              v-for="blk in statusBlocks"
              :key="blk.label"
              class="pd-status-card"
              :class="{
                'pd-status-card--clickable': blk.canAdvance,
                'pd-status-card--disabled': blk.isDisabled && !blk.canAdvance,
                'pd-status-card--busy': blk.isBusy,
              }"
              :style="{
                borderColor: statusColorMap[blk.status],
                backgroundColor: statusColorMap[blk.status] + '10',
              }"
              :title="
                blk.isDisabled ? '已完成' : blk.canAdvance ? '点击后推进状态' : '暂无权限或暂不可用'
              "
              @click="handleStatusCardClick(blk)"
            >
              <div class="pd-status-card__label">{{ blk.label }}</div>
              <div
                class="pd-status-card__state-pill"
                :style="{
                  color: statusColorMap[blk.status],
                  backgroundColor: statusColorMap[blk.status] + '12',
                }"
              >
                {{ blk.stateText }}
              </div>
              <div class="pd-status-card__members">
                <template v-if="blk.members && blk.members.length > 0">
                  {{ blk.members.join('、') }}
                </template>
                <template v-else>未分配</template>
              </div>
              <div v-if="blk.isBusy" class="pd-status-card__busy">推进中...</div>
            </div>
          </div>
        </div>

        <!-- 指标统计 -->
        <div class="pd-metrics-card">
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
        </div>

        <!-- 标记分布图表 -->
        <!-- <div class="pd-chart-card">
          <h2 class="pd-section-title">每页标记分布</h2>
          <div class="pd-chart-canvas" :style="{ height: chartDynamicHeight + 'px' }">
            <div v-if="loadingMarkers" class="pd-chart-loading">
              <LoadingCircle />
            </div>
            <div v-else-if="loadingMarkersFailed" class="pd-chart-empty pd-chart-error">
              加载失败
            </div>
            <Line
              v-else-if="chartData.labels.length > 0"
              :data="chartData"
              :options="chartOptions"
            />
            <div v-else class="pd-chart-empty">暂无标记数据</div>
          </div>
        </div> -->
        <!-- 预览图（使用 Moetran 的 cover_url，分页显示，低分辨率不走缓存） -->
        <div class="pd-preview-card">
          <h2 class="pd-section-title">预览</h2>
          <div v-if="!primaryFiles.length" class="pd-preview-empty">
            <template v-if="loadingMarkers || measuringPreviews || loadingPreview">
              <LoadingCircle />
            </template>
            <template v-else>暂无预览</template>
          </div>
          <div v-else>
            <div class="pd-preview-grid" ref="previewGridRef" :style="previewGridStyle">
              <div v-for="f in renderPreviewFiles" :key="f.id" class="pd-preview-item">
                <div v-if="previewImageMap[f.id]">
                  <img :src="previewImageMap[f.id]" :alt="f.name" />
                </div>
                <div v-else class="pd-preview-placeholder">
                  {{ !measuringPreviews && !loadingPreview ? '暂无预览' : '' }}
                </div>
              </div>
            </div>
            <div v-if="measuringPreviews || loadingPreview" class="pd-preview-overlay">
              <LoadingCircle />
            </div>
            <div class="pd-preview-pager">
              <button
                class="pd-btn pd-btn--secondary"
                @click="goPrevPreview"
                :disabled="previewPage <= 1"
                type="button"
              >
                上一页
              </button>
              <span class="pd-preview-page">{{ previewPage }} / {{ totalPreviewPages }}</span>
              <button
                class="pd-btn pd-btn--secondary"
                @click="goNextPreview"
                :disabled="previewPage >= totalPreviewPages"
                type="button"
              >
                下一页
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="statusConfirm.visible" class="pd-dialog-mask" @click.self="closeStatusConfirm">
      <div class="pd-dialog">
        <div class="pd-dialog__title">推进状态</div>
        <div class="pd-dialog__body">
          是否要将{{ statusConfirm.label }}状态推进为“{{ statusConfirmTargetName }}”？
        </div>
        <div class="pd-dialog__actions">
          <button
            type="button"
            class="pd-btn pd-btn--secondary"
            @click="closeStatusConfirm"
            :disabled="confirmDialogBusy"
          >
            取消
          </button>
          <button
            type="button"
            class="pd-btn pd-btn--primary"
            @click="confirmStatusAdvance"
            :disabled="confirmDialogBusy"
          >
            {{ confirmDialogBusy ? '执行中...' : '确认推进' }}
          </button>
        </div>
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
.pd-tag--team {
  background: #f0f8ff;
  border-color: rgba(118, 184, 255, 0.5);
  color: #2f5a8f;
  font-weight: 600;
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
.pd-btn--danger {
  border-color: rgba(210, 120, 120, 0.7);
  color: #823c3c;
  background: #fff3f3;
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

/* === 主布局：左列封面 + 右列内容 === */
.pd-main-layout {
  display: flex;
  gap: 18px;
  align-items: flex-start;
}

/* 左列：封面 */
.pd-cover-column {
  flex: 0 0 180px;
  width: 180px;
}
.pd-card__cover {
  width: 180px;
  height: 250px;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: 0 6px 20px rgba(140, 180, 230, 0.16);
}
.pd-cover-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.pd-cover-placeholder {
  font-size: 13px;
  color: #7a8fa5;
}
.pd-cover-loading {
  display: flex;
  align-items: center;
  justify-content: center;
}

.pd-cover-actions {
  margin-top: 10px;
  width: 100%;
  display: flex;
  justify-content: center;
}
.pd-cover-action-button {
  width: 100%;
  max-width: 180px;
  box-sizing: border-box;
}

/* 右列：内容区 */
.pd-content-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

/* 通用卡片样式 */
.pd-overview-card,
.pd-metrics-card,
.pd-chart-card {
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 14px;
  padding: 12px 16px;
  box-shadow: 0 6px 18px rgba(140, 180, 230, 0.16);
}

/* 章节标题 */
.pd-section-title {
  margin: 0 0 10px 0;
  font-size: 16px;
  font-weight: 600;
  color: #23415b;
}
.pd-status-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
.pd-status-card {
  background: #f8fbff;
  border: 2px solid;
  border-radius: 10px;
  padding: 8px 10px;
  text-align: center;
  box-shadow: 0 4px 12px rgba(140, 180, 230, 0.12);
  cursor: default;
  position: relative;
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease;
}
.pd-status-card--clickable {
  cursor: pointer;
}
.pd-status-card--clickable:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 18px rgba(140, 180, 230, 0.24);
}
.pd-status-card--disabled {
  opacity: 0.55;
  pointer-events: none;
}
.pd-status-card--busy {
  pointer-events: none;
}
.pd-status-card__label {
  font-size: 12px;
  font-weight: 600;
  color: #2a4a63;
  margin-bottom: 4px;
}
.pd-status-card__state-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56px;
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  color: #2f5a8f;
  background: rgba(47, 90, 143, 0.12);
  margin-bottom: 6px;
}
.pd-status-card__members {
  font-size: 11px;
  color: #4a5f7a;
  line-height: 1.4;
}
.pd-status-card__busy {
  margin-top: 6px;
  font-size: 11px;
  color: #2f5a8f;
}
.pd-metrics-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
.pd-metric-box {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: #f8fbff;
  border: 1px solid rgba(150, 180, 210, 0.3);
  padding: 10px 12px;
  border-radius: 10px;
  box-shadow: 0 4px 12px rgba(140, 180, 230, 0.12);
  transition: background 0.2s ease;
}
.pd-metric-box:hover {
  background: #eef6ff;
}
.pd-metric-box__label {
  font-size: 11px;
  color: #5b768e;
}
.pd-metric-box__value {
  font-size: 20px;
  font-weight: 600;
  color: #23415b;
}
.pd-metric-box__value--yellow {
  color: #b98119;
}
.pd-metric-box__value--pink {
  color: #b24874;
}

/* Chart block adjustments */
.pd-chart-canvas {
  background: #f8fbff;
  border: 1px solid rgba(150, 180, 210, 0.3);
  border-radius: 10px;
  padding: 12px 14px;
  box-shadow: 0 4px 12px rgba(140, 180, 230, 0.12);
  box-sizing: border-box;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}
.pd-chart-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}
.pd-chart-empty {
  font-size: 13px;
  color: #6b859d;
}
.pd-chart-error {
  color: #c62828;
  font-weight: 500;
}

.pd-preview-card {
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 12px;
  padding: 12px 14px;
  box-shadow: 0 6px 18px rgba(140, 180, 230, 0.12);
  position: relative;
}
.pd-preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 10px;
}
.pd-preview-item {
  width: 100%;
  height: 120px;
  background: #f8fafc;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}
.pd-preview-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.pd-preview-pager {
  margin-top: 10px;
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: center;
}
.pd-preview-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 12px;
  pointer-events: none;
  z-index: 6;
}
.pd-preview-page {
  font-size: 13px;
  color: #4a5f7a;
  min-width: 64px;
  text-align: center;
}
.pd-preview-empty {
  font-size: 13px;
  color: #7a8fa5;
  padding: 12px 0;
}
.pd-preview-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #7a8fa5;
  font-size: 13px;
}

.pd-dialog-mask {
  position: fixed;
  inset: 0;
  background: rgba(17, 35, 64, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 60;
}
.pd-dialog {
  width: 320px;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 14px;
  box-shadow: 0 16px 32px rgba(30, 60, 100, 0.2);
  padding: 18px 20px;
}
.pd-dialog__title {
  font-size: 16px;
  font-weight: 600;
  color: #23415b;
  margin-bottom: 10px;
}
.pd-dialog__body {
  font-size: 14px;
  color: #445b72;
  line-height: 1.6;
  margin-bottom: 16px;
}
.pd-dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

@media (max-width: 1100px) {
  .pd-main-layout {
    flex-direction: column;
  }
  .pd-cover-column {
    width: 100%;
    display: flex;
    justify-content: center;
  }
  .pd-status-cards {
    grid-template-columns: repeat(2, 1fr);
  }
  .pd-metrics-grid {
    grid-template-columns: repeat(2, 1fr);
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
