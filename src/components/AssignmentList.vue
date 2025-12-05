<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { getAssignments } from '../ipc/project';
import type { ResAssignment } from '../api/model/assignment';
import { useToastStore } from '../stores/toast';

// 时间区域定义
type TimeRange = '1day' | '1week' | '1month';

interface TimeSection {
  label: string;
  range: TimeRange;
  timeStart: number;
  assignments: ResAssignment[];
}

const emit = defineEmits<{
  (e: 'view-change', view: 'projects' | 'assignments'): void;
}>();

const props = defineProps<{
  teamId?: string | null;
  currentView?: 'projects' | 'assignments';
}>();

const toastStore = useToastStore();
// 视图模式：取自父级 currentView
const viewMode = computed(() => props.currentView ?? 'assignments');

// 无 teamId 时禁用“派活列表”按钮
const canViewAssignments = computed(() => !!props.teamId);

function switchView(view: 'projects' | 'assignments'): void {
  emit('view-change', view);
}

// 当前选择的时间范围
const selectedTimeRange = ref<TimeRange>('1day');
// 所有 assignments 数据
const allAssignments = ref<ResAssignment[]>([]);
const isLoading = ref(false);

// 根据选择的时间范围计算 timeStart
const currentTimeStart = computed(() => {
  const now = Math.floor(Date.now() / 1000);
  switch (selectedTimeRange.value) {
    case '1day':
      return now - 86400;
    case '1week':
      return now - 604800;
    case '1month':
      return now - 2592000;
  }
});

// 时间区域：按近到远排列
const timeSections = computed<TimeSection[]>(() => {
  const now = Math.floor(Date.now() / 1000);
  const sections: TimeSection[] = [
    {
      label: '近一天',
      range: '1day',
      timeStart: now - 86400,
      assignments: [],
    },
    {
      label: '近一周',
      range: '1week',
      timeStart: now - 604800,
      assignments: [],
    },
    {
      label: '近一个月',
      range: '1month',
      timeStart: now - 2592000,
      assignments: [],
    },
  ];

  // 将 assignments 分配到对应的时间区域
  for (const assign of allAssignments.value) {
    // 从最近的时间区域开始匹配
    for (const section of sections) {
      if (assign.updatedAt >= section.timeStart) {
        section.assignments.push(assign);
        break;
      }
    }
  }

  // 仅返回包含数据的时间区域
  return sections.filter(s => s.assignments.length > 0);
});

// 当前页显示的时间区域（用于分页）
const currentPageSections = ref<TimeSection[]>([]);
const currentPage = ref(1);

// 分页逻辑：计算当前页应显示的时间区域
function updatePageSections(): void {
  // 简化逻辑：每页显示一个时间区域
  const pageIndex = currentPage.value - 1;
  if (pageIndex >= 0 && pageIndex < timeSections.value.length) {
    currentPageSections.value = [timeSections.value[pageIndex]];
  } else {
    currentPageSections.value = [];
  }
}

watch(
  () => timeSections.value,
  () => {
    updatePageSections();
  },
  { immediate: true }
);

watch(currentPage, () => {
  updatePageSections();
});

// 是否有上一页/下一页
const hasPrevPage = computed(() => currentPage.value > 1);
const hasNextPage = computed(() => currentPage.value < timeSections.value.length);

function goPrevPage(): void {
  if (hasPrevPage.value) {
    currentPage.value--;
  }
}

function goNextPage(): void {
  if (hasNextPage.value) {
    currentPage.value++;
  }
}

// 获取 assignments 数据
async function fetchAssignments(): Promise<void> {
  isLoading.value = true;
  try {
    const data = await getAssignments(currentTimeStart.value);
    allAssignments.value = data;
  } catch (err) {
    console.error('[AssignmentList] 获取派活列表失败:', err);
    toastStore.show(`获取派活列表失败：${String(err)}`);
    allAssignments.value = [];
  } finally {
    isLoading.value = false;
  }
}

// 切换时间范围
function selectTimeRange(range: TimeRange): void {
  selectedTimeRange.value = range;
  currentPage.value = 1;
  void fetchAssignments();
}

// 格式化角色标签
function formatRoles(assign: ResAssignment): string {
  const roles: string[] = [];
  if (assign.isTranslator) roles.push('翻译');
  if (assign.isProofreader) roles.push('校对');
  if (assign.isTypesetter) roles.push('嵌字');
  return roles.join(' / ') || '无';
}

// 格式化时间
function formatTime(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString('zh-CN');
}

// 初始化加载
onMounted(() => {
  void fetchAssignments();
});

// 当 teamId 变化时重新加载
watch(
  () => props.teamId,
  () => {
    currentPage.value = 1;
    void fetchAssignments();
  }
);
</script>

<template>
  <section class="assignment-list">
    <header class="assignment-list__header">
      <div class="assignment-list__view-toggle view-toggle">
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
      <div class="assignment-list__controls">
        <!-- 时间范围选择 -->
        <div class="time-range-btns">
          <button
            type="button"
            class="time-range-btn"
            :class="{ 'time-range-btn--active': selectedTimeRange === '1day' }"
            @click="selectTimeRange('1day')"
            :disabled="isLoading"
          >
            近一天
          </button>
          <button
            type="button"
            class="time-range-btn"
            :class="{ 'time-range-btn--active': selectedTimeRange === '1week' }"
            @click="selectTimeRange('1week')"
            :disabled="isLoading"
          >
            近一周
          </button>
          <button
            type="button"
            class="time-range-btn"
            :class="{ 'time-range-btn--active': selectedTimeRange === '1month' }"
            @click="selectTimeRange('1month')"
            :disabled="isLoading"
          >
            近一个月
          </button>
        </div>

        <!-- 分页控制 -->
        <div class="pagination-controls">
          <button
            type="button"
            class="pagination-btn"
            @click="goPrevPage"
            :disabled="!hasPrevPage || isLoading"
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
          <span class="page-indicator">{{ currentPage }} / {{ timeSections.length || 1 }}</span>
          <button
            type="button"
            class="pagination-btn"
            @click="goNextPage"
            :disabled="!hasNextPage || isLoading"
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

    <div class="assignment-list__content">
      <div v-if="isLoading" class="assignment-list__loading">加载中...</div>
      <div v-else-if="!currentPageSections.length" class="assignment-list__empty">暂无派活</div>
      <div v-else class="assignment-list__timeline">
        <div v-for="section in currentPageSections" :key="section.range" class="timeline-section">
          <div class="timeline-label">{{ section.label }}</div>
          <div class="timeline-items">
            <div
              v-for="assign in section.assignments"
              :key="`${assign.projId}-${assign.memberId}`"
              class="assign-card"
            >
              <div class="assign-card__primary">
                <div class="assign-card__left">
                  <div class="assign-card__roles">{{ formatRoles(assign) }}</div>
                  <div class="assign-card__member">{{ assign.username }}</div>
                </div>
                <div class="assign-card__time">
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <circle cx="12" cy="12" r="10"></circle>
                    <polyline points="12,6 12,12 16,14"></polyline>
                  </svg>
                  {{ formatTime(assign.updatedAt) }}
                </div>
              </div>
              <div class="assign-card__proj">
                【{{ assign.projsetSerial }}-{{ assign.projsetIndex }}】{{ assign.projName }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.assignment-list {
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

.assignment-list__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

/* 视图切换按钮样式（与 ProjectList 保持一致） */
.view-toggle {
  display: flex;
  gap: 0;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid rgba(118, 184, 255, 0.35);
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
  border-right: 1px solid rgba(118, 184, 255, 0.2);
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

.assignment-list__controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.time-range-btns {
  display: flex;
  gap: 8px;
}

.time-range-btn {
  padding: 6px 12px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    background 0.12s ease,
    box-shadow 0.12s ease,
    border-color 0.12s ease;
}

.time-range-btn:hover:not(:disabled) {
  background: #eaf6ff;
  box-shadow: 0 4px 12px rgba(118, 184, 255, 0.08);
  transform: translateY(-1px);
}

.time-range-btn--active {
  background: linear-gradient(135deg, #5ba3e0, #6db4f0);
  color: #fff;
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 4px 12px rgba(118, 184, 255, 0.25);
}

.time-range-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pagination-btn {
  padding: 4px 6px;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    background 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.15s ease;
}

.pagination-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination-btn:not(:disabled):hover {
  background: #eef6ff;
  box-shadow: 0 4px 14px rgba(118, 184, 255, 0.25);
  transform: translateY(-1px);
}

.page-indicator {
  font-size: 12px;
  font-weight: 600;
  color: #2a4f7a;
}

.assignment-list__content {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
}

.assignment-list__loading,
.assignment-list__empty {
  padding: 40px 0;
  text-align: center;
  font-size: 14px;
  color: #4a5f7d;
}

.assignment-list__timeline {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.timeline-section {
  display: grid;
  grid-template-columns: 120px 1fr;
  gap: 16px;
  align-items: start;
}

.timeline-label {
  position: sticky;
  top: 16px;
  font-size: 16px;
  font-weight: 600;
  color: #2f5a8f;
  padding: 8px 12px;
  background: rgba(240, 248, 255, 0.95);
  border-radius: 10px;
  border: 1px solid rgba(118, 184, 255, 0.25);
  text-align: center;
}

.timeline-items {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}

.assign-card {
  padding: 10px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.98);
  border: 1px solid rgba(210, 220, 235, 0.65);
  box-shadow: 0 4px 10px rgba(132, 166, 212, 0.08);
  transition:
    box-shadow 0.14s ease,
    transform 0.14s ease,
    border-color 0.14s ease;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.assign-card:hover {
  box-shadow: 0 8px 20px rgba(132, 166, 212, 0.18);
  border-color: rgba(118, 184, 255, 0.6);
  transform: translateY(-2px);
}

.assign-card__proj {
  font-size: 12px;
  font-weight: 600;
  color: #294061;
  line-height: 1.4;
  word-break: break-word;
}

.assign-card__primary {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.assign-card__left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.assign-card__member {
  font-size: 14px;
  color: #4a5f7d;
  font-weight: 600;
}

.assign-card__roles {
  font-size: 14px;
  font-weight: 600;
  color: #5ba3e0;
  padding: 4px 8px;
  background: rgba(210, 236, 255, 0.8);
  border-radius: 6px;
  text-align: center;
}

.assign-card__time {
  font-size: 10px;
  color: #6b7c8f;
  display: flex;
  align-items: center;
  gap: 4px;
}
</style>
