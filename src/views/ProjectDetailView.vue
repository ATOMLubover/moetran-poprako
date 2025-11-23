<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
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
import CircularProgress from '../components/CircularProgress.vue';
import { useToastStore } from '../stores/toast';

// TODO: 后续接口可能扩展字段（如权限、进度来源），保持结构可扩展
interface PageMarkerData {
  // 单页标记统计
  pageNumber: number;
  inBoxMarkers: number;
  outOfBoxMarkers: number;
}

// TODO: 状态枚举可能会增加（如 paused / archived）
type ProjectStatus = 'pending' | 'in_progress' | 'completed';

// TODO: 项目详情字段未来可能会增加 Poprako 相关统计 / 团队角色等
interface ProjectDetail {
  id: number;
  title: string;
  authorName: string;
  uploader: string;
  description: string;
  worksetId: number;
  worksetIndex: number;
  totalPages: number;
  totalMarkers: number;
  totalTranslatedMarkers: number;
  totalProofreadMarkers: number;
  translationStatus: ProjectStatus;
  proofreadingStatus: ProjectStatus;
  letteringStatus: ProjectStatus;
  reviewingStatus: ProjectStatus;
  allowAutoJoin: boolean;
  isHidden: boolean;
  translators: string[];
  proofreaders: string[];
  letterers: string[];
  reviewers: string[];
  pageMarkers: PageMarkerData[];
}

const props = defineProps<{ projectId: number }>();

const emit = defineEmits<{ (e: 'close'): void; (e: 'open-translator', enabled: boolean): void }>();

const toastStore = useToastStore();

// 项目详情数据
const project = ref<ProjectDetail | null>(null);

// 修改面板显隐（后续可拆分成独立子组件）
const showModifier = ref(false);

// 详细标记数据加载状态
const loadingMarkers = ref(false);
// 卡片容器引用（用于后续可能的动态高度微调）
const cardRef = ref<HTMLElement | null>(null);

// 是否为项目管理员（后续可根据真实用户身份判断）
const isMeProjectMgr = computed(() => true); // TODO: 替换为真实权限判断

// 是否在项目中（后续根据服务端返回的成员列表判断）
const isMeInProject = computed(() => true); // TODO: 替换为真实参与判断

// --- 进度相关计算 ---
const translationProgress = computed(() => {
  if (!project.value || project.value.totalMarkers === 0) return 0;
  return (project.value.totalTranslatedMarkers / project.value.totalMarkers) * 100;
});

const proofreadProgress = computed(() => {
  if (!project.value || project.value.totalMarkers === 0) return 0;
  return (project.value.totalProofreadMarkers / project.value.totalMarkers) * 100;
});

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
        label: '框内标记',
        data: project.value.pageMarkers.map(p => p.inBoxMarkers),
        borderColor: '#3d79c4',
        backgroundColor: 'rgba(61,121,196,0.25)',
        tension: 0.28,
        pointRadius: 3.6,
        fill: true,
      },
      {
        label: '框外标记',
        data: project.value.pageMarkers.map(p => p.outOfBoxMarkers),
        borderColor: '#cc4d7e',
        backgroundColor: 'rgba(204,77,126,0.25)',
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

// --- Mock 数据获取 ---
// 模拟获取项目概要（必须以 __mock 开头）
async function __mockFetchProjectSummary(): Promise<ProjectDetail> {
  // 获取项目概要
  await new Promise(r => setTimeout(r, 420));

  return {
    id: props.projectId,
    title: '調整記録 ご主人様の元へ届くまで',
    authorName: 'しぷおる',
    uploader: '杨声器不成器',
    description: '',
    worksetId: 1024,
    worksetIndex: 5,
    totalPages: 28,
    totalMarkers: 410,
    totalTranslatedMarkers: 180,
    totalProofreadMarkers: 95,
    translationStatus: 'completed',
    proofreadingStatus: 'in_progress',
    letteringStatus: 'pending',
    reviewingStatus: 'pending',
    allowAutoJoin: false,
    isHidden: true,
    translators: ['Hatsu1ki', '电容'],
    proofreaders: ['Parody'],
    letterers: ['水月', '鸟龙'],
    reviewers: ['夏目历'],
    pageMarkers: [],
  };
}

// 模拟获取每页标记分布（必须以 __mock 开头）
async function __mockFetchMarkerDetails(totalPages: number): Promise<PageMarkerData[]> {
  // 获取标记分布
  loadingMarkers.value = true;

  await new Promise(r => setTimeout(r, 620));

  const list: PageMarkerData[] = [];

  for (let i = 1; i <= totalPages; i++) {
    list.push({
      pageNumber: i,
      inBoxMarkers: Math.floor(Math.random() * 12) + 1,
      outOfBoxMarkers: Math.floor(Math.random() * 6),
    });
  }

  loadingMarkers.value = false;

  return list;
}

// 加载项目详情与标记分布
async function loadProject(): Promise<void> {
  // 加载项目整体数据
  try {
    const summary = await __mockFetchProjectSummary();

    project.value = summary;

    const markers = await __mockFetchMarkerDetails(summary.totalPages);

    if (project.value) {
      project.value.pageMarkers = markers;
    }
  } catch (err) {
    toastStore.show('加载项目详情失败', 'error');
  }
}

// 切换修改面板显隐
function handleToggleModifier(): void {
  // 切换修改面板
  showModifier.value = !showModifier.value;
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

// 进入翻译工作台（带参与者模式或只读模式）
function openTranslator(enabled: boolean): void {
  // 打开翻译工作台
  emit('open-translator', enabled);
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

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
});
</script>

<template>
  <section class="project-detail" v-if="project">
    <header class="pd-header">
      <div class="pd-header__left">
        <h1 class="pd-title">{{ project.title }}</h1>
        <p class="pd-meta">画师：{{ project.authorName }} / 上传者：{{ project.uploader }}</p>
      </div>
      <div class="pd-header__right">
        <button type="button" class="pd-close" @click="handleClose" title="关闭">×</button>
      </div>
    </header>

    <div class="pd-topbar">
      <div class="pd-tags">
        <span class="pd-tag">作品集 ID: {{ project.worksetId }}</span>
        <span class="pd-tag">项目 ID: {{ project.worksetIndex }}</span>
        <span
          class="pd-tag"
          :class="{ 'pd-tag--ok': project.allowAutoJoin, 'pd-tag--bad': !project.allowAutoJoin }"
        >
          {{ project.allowAutoJoin ? '允许自动加入' : '禁止自动加入' }}
        </span>
        <span v-if="project.isHidden" class="pd-tag pd-tag--warn">隐藏项目</span>
      </div>
      <div class="pd-actions">
        <button
          v-if="isMeProjectMgr"
          type="button"
          class="pd-btn pd-btn--primary"
          @click="handleToggleModifier"
        >
          {{ showModifier ? '取消修改' : '修改项目' }}
        </button>
        <button v-else type="button" class="pd-btn pd-btn--primary" @click="handleJoinOrLeave">
          {{ isMeInProject ? '退出项目' : '加入项目' }}
        </button>
        <button type="button" class="pd-btn pd-btn--secondary" @click="openTranslator(true)">
          参与者翻译
        </button>
        <button type="button" class="pd-btn pd-btn--secondary" @click="openTranslator(false)">
          只读翻译
        </button>
      </div>
    </div>

    <div class="pd-description">
      <h2 class="pd-subtitle">描述</h2>
      <p class="pd-desc-text">
        {{ project.description || '（暂无描述）' }}
      </p>
    </div>

    <div class="pd-card" ref="cardRef">
      <div class="pd-card__inner">
        <div class="pd-card__left">
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
        </div>
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
.project-detail {
  display: flex;
  flex-direction: column;
  padding: 28px 38px 80px;
  box-sizing: border-box;
  min-height: 100vh;
  background: linear-gradient(180deg, #f5f9ff 0%, #ffffff 65%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
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
  margin-top: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
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

.pd-description {
  margin-top: 22px;
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
  margin-top: 28px;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.45);
  border-radius: 18px;
  padding: 26px 30px;
  box-shadow: 0 14px 42px rgba(140, 180, 230, 0.22);
}
.pd-card__inner {
  display: flex;
  align-items: stretch;
  gap: 32px;
}
.pd-card__left {
  flex: 0 0 170px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 34px;
  padding: 4px 0 8px;
  border-right: 1px solid rgba(150, 180, 210, 0.35);
  box-sizing: border-box;
}
.pd-card__right {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 24px;
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
  gap: 14px;
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
  gap: 12px;
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
  gap: 18px;
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
  margin-top: 34px;
}
.pd-chart-canvas {
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.45);
  border-radius: 18px;
  padding: 20px 24px;
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
    padding: 24px 24px 72px;
  }
}
</style>
