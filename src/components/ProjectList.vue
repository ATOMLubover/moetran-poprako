<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import type {
  ProjectBasicInfo,
  WorkPhase,
  PhaseStatus,
  PhaseChip,
} from '../api/model/displayProject';
import { useToastStore } from '../stores/toast';

// 使用共享类型定义（见 src/api/model/displayProject.ts）

// 组件事件：打开详情 / 创建项目
const emit = defineEmits<{
  (e: 'open-detail', projectId: number): void;
  (e: 'create'): void;
}>();

// Toast store
const toastStore = useToastStore();

// 内部项目列表数据（若父组件未传入则使用本地加载）
const internalProjects = ref<ProjectBasicInfo[]>([]);

// 放宽为 any[] 以兼容父组件不同的展示数据结构，避免 props 类型不匹配错误
// 接收外部传入的项目数据（用于仪表盘过滤后的结果展示）
// 使用共享的强类型接口，避免使用 any
const props = defineProps<{ projects?: ProjectBasicInfo[] }>();

// 是否正在加载标记
const isLoading = ref(false);

// ======================= Mock 数据生成 =======================
// 生成单个项目的阶段初始状态
function createPhaseSet(seed: number): PhaseChip[] {
  // 使用 seed 简单决定不同阶段的状态分布，形成视觉差异
  const phases: WorkPhase[] = ['translate', 'proof', 'typeset', 'review', 'publish'];

  return phases.map((phase, i) => {
    const rotate = (seed + i) % 5;

    let status: PhaseStatus = 'unset';

    if (rotate === 1) status = 'pending';
    else if (rotate === 2) status = 'wip';
    else if (rotate === 3) status = 'completed';
    else status = 'unset';

    const labelMap: Record<WorkPhase, string> = {
      translate: '翻译',
      proof: '校对',
      typeset: '嵌字',
      review: '监修',
      publish: '发布',
    };

    return {
      phase,
      status,
      label: labelMap[phase],
    } satisfies PhaseChip;
  });
}

// Mock 拉取项目列表
async function __mockFetchProjects(): Promise<ProjectBasicInfo[]> {
  // 真实实现应调用后端接口
  return new Promise(resolve => {
    const base: ProjectBasicInfo[] = [
      { id: 101, index: 1, author: '作者A', title: '第一个项目标题', phases: createPhaseSet(3) },
      { id: 102, index: 2, author: '作者B', title: '第二个项目标题', phases: createPhaseSet(5) },
      { id: 103, index: 3, author: '作者C', title: '第三个项目标题', phases: createPhaseSet(9) },
      { id: 104, index: 4, author: '作者D', title: '第四个项目标题', phases: createPhaseSet(11) },
    ];

    setTimeout(() => resolve(base), 160);
  });
}

// ======================= 行为函数 =======================
// 加载项目列表
async function loadProjects(): Promise<void> {
  isLoading.value = true;

  try {
    const list = await __mockFetchProjects();

    internalProjects.value = list;
  } catch (err) {
    toastStore.show('加载项目失败', 'error');
  } finally {
    isLoading.value = false;
  }
}

// 点击详情
function handleOpenDetail(projectId: number): void {
  emit('open-detail', projectId);
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

// 是否为空列表
// 最终展示数据：优先使用外部传入，其次内部加载
const displayProjects = computed<ProjectBasicInfo[]>(() =>
  props.projects && props.projects.length > 0 ? props.projects : internalProjects.value
);

const isEmpty = computed(() => !isLoading.value && displayProjects.value.length === 0);

// 当外部 projects 变化时重新计算空状态
watch(
  () => props.projects,
  () => {
    // 若外部传入空数组且未在加载中则显示为空
  },
  { deep: true }
);

onMounted(() => {
  loadProjects();
});
</script>

<template>
  <section class="project-list">
    <header class="project-list__header">
      <h3 class="project-list__title">当前项目</h3>
      <button
        type="button"
        class="project-list__create"
        @click="handleCreateProject"
        :disabled="isLoading"
      >
        创建新项目
      </button>
    </header>

    <div class="project-list__content" v-if="!isEmpty">
      <ul class="project-list__items" v-if="displayProjects.length > 0">
        <li v-for="item in displayProjects" :key="item.id" class="project-list__item">
          <div class="project-list__item-main">
            <h3 class="project-list__item-title">
              【 {{ item.id }} - {{ item.index }} 】[{{ item.author }}] {{ item.title }}
            </h3>
            <div class="project-list__chips">
              <span v-for="phase in item.phases" :key="phase.phase" :class="chipClass(phase)">
                {{ phase.label }}
              </span>
            </div>
          </div>
          <div class="project-list__actions">
            <button
              type="button"
              class="project-list__detail-btn"
              @click="handleOpenDetail(item.id)"
            >
              详情
            </button>
          </div>
        </li>
      </ul>
      <div v-if="isLoading" class="project-list__loading">加载中...</div>
    </div>
    <div v-else class="project-list__empty">暂无项目</div>
  </section>
</template>

<style scoped>
.project-list {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 24px 30px 28px;
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

.project-list__title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: #1f2e43;
}

.project-list__create {
  border: none;
  border-radius: 999px;
  padding: 10px 22px;
  font-size: 14px;
  font-weight: 600;
  background: linear-gradient(135deg, rgba(177, 207, 239, 0.95), rgba(160, 206, 255, 0.9));
  color: #10395d;
  cursor: pointer;
  box-shadow: 0 10px 22px rgba(120, 170, 230, 0.32);
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.project-list__create:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}

.project-list__create:not(:disabled):hover {
  transform: translateY(-2px);
  box-shadow: 0 14px 30px rgba(118, 184, 255, 0.42);
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
  gap: 16px;
}

.project-list__item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 24px;
  padding: 16px 18px 14px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.96);
  border: 1px solid rgba(210, 220, 235, 0.7);
  box-shadow: 0 10px 20px rgba(132, 166, 212, 0.16);
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease,
    border-color 0.18s ease;
}

.project-list__item:hover {
  box-shadow: 0 16px 32px rgba(132, 166, 212, 0.22);
  border-color: rgba(118, 184, 255, 0.85);
  transform: translateY(-2px);
}

.project-list__item-title {
  margin: 0 0 12px 0;
  font-size: 15px;
  font-weight: 600;
  color: #294061;
  letter-spacing: 0.4px;
}

.project-list__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 10px;
  align-items: center;
}

.project-list__chip {
  padding: 4px 12px 5px;
  border-radius: 999px;
  font-size: 12px;
  letter-spacing: 0.3px;
  font-weight: 600;
  line-height: 1;
  border: 1px solid transparent;
  background: rgba(240, 246, 255, 0.85);
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
  border: none;
  border-radius: 12px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  background: linear-gradient(135deg, rgba(124, 205, 182, 0.95), rgba(146, 214, 222, 0.9));
  color: #1c3f56;
  transition: transform 0.18s ease;
}

.project-list__detail-btn:hover {
  transform: translateY(-2px);
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
