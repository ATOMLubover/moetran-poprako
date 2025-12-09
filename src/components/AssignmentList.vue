<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import type { ResAssignment } from '../api/model/assignment';
import { useToastStore } from '../stores/toast';
import { getAssignments } from '../ipc/project';

// 时间区域定义
type TimeRange = '1day' | '1week' | '1month';

// 每个分页中的时间段及其 assigns
interface PageSection {
  label: string;
  range: TimeRange;
  assignments: ResAssignment[];
}

// 一页的数据结构（PageData 不再单独声明，直接使用 PageSection[]）

const emit = defineEmits<{
  (e: 'view-change', view: 'projects' | 'assignments' | 'members'): void;
}>();

const props = defineProps<{
  teamId?: string | null;
  currentView?: 'projects' | 'assignments' | 'members';
}>();

const toastStore = useToastStore();
// 视图模式：取自父级 currentView
const viewMode = computed(() => props.currentView ?? 'assignments');

// 无 teamId 时禁用“派活列表”按钮
const canViewAssignments = computed(() => !!props.teamId);

function switchView(view: 'projects' | 'assignments' | 'members'): void {
  emit('view-change', view);
}

// 当前选择的时间范围
const selectedTimeRange = ref<TimeRange>('1day');
// 所有 assignments 数据
const allAssignments = ref<ResAssignment[]>([]);
const isLoading = ref(false);

// 根据选择的时间范围计算 timeStart（暂时保留，用于后续真实API调用）
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

// 时间区域定义（分段显示）
// interface TimeSection {
//   label: string;
//   range: TimeRange;
//   timeStart: number;
//   timeEnd: number;
//   assignments: ResAssignment[];
// }

// 每页显示的最大 assign 数量（增大以充分利用空间）
const pageSize = 50;

// 根据当前选择的时间范围，将 assigns 分到各时间段（不重叠）
const categorizedAssignments = computed(() => {
  const now = Math.floor(Date.now() / 1000);

  // 定义时间段边界
  const day1 = now - 86400;
  const week1 = now - 604800;
  const month1 = now - 2592000;

  // 根据选择的时间范围决定显示哪些时间段
  const result: { label: string; range: TimeRange; assigns: ResAssignment[] }[] = [];

  // 近一天的 assigns（按时间升序）
  const dayAssigns = allAssignments.value
    .filter(a => a.updatedAt >= day1)
    .slice()
    .sort((x, y) => x.updatedAt - y.updatedAt)
    .reverse();

  // 近一周（不含近一天）的 assigns（按时间升序）
  const weekAssigns = allAssignments.value
    .filter(a => a.updatedAt >= week1 && a.updatedAt < day1)
    .slice()
    .sort((x, y) => x.updatedAt - y.updatedAt)
    .reverse();

  // 近一个月（不含近一周）的 assigns（按时间升序）
  const monthAssigns = allAssignments.value
    .filter(a => a.updatedAt >= month1 && a.updatedAt < week1)
    .slice()
    .sort((x, y) => x.updatedAt - y.updatedAt)
    .reverse();

  // 根据选择的时间范围依次添加
  if (dayAssigns.length > 0) {
    result.push({ label: '近一天', range: '1day', assigns: dayAssigns });
  }

  if (selectedTimeRange.value === '1week' || selectedTimeRange.value === '1month') {
    if (weekAssigns.length > 0) {
      result.push({ label: '近一周', range: '1week', assigns: weekAssigns });
    }
  }

  if (selectedTimeRange.value === '1month') {
    if (monthAssigns.length > 0) {
      result.push({ label: '近一个月', range: '1month', assigns: monthAssigns });
    }
  }

  return result;
});

// 将所有 assigns 按时间段顺序平铺，并记录每个 assign 所属的时间段
const flatAssignments = computed(() => {
  const result: { assign: ResAssignment; label: string; range: TimeRange }[] = [];
  for (const category of categorizedAssignments.value) {
    for (const assign of category.assigns) {
      result.push({ assign, label: category.label, range: category.range });
    }
  }
  return result;
});

// 总页数
const totalPages = computed(() => {
  return Math.ceil(flatAssignments.value.length / pageSize) || 1;
});

// 当前页显示的内容（包含多个时间段）
const currentPageSections = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  const end = Math.min(start + pageSize, flatAssignments.value.length);
  const pageItems = flatAssignments.value.slice(start, end);

  // 将当前页的 items 按时间段分组
  const sections: PageSection[] = [];
  let currentSection: PageSection | null = null;

  for (const item of pageItems) {
    if (!currentSection || currentSection.range !== item.range) {
      // 新的时间段
      currentSection = {
        label: item.label,
        range: item.range,
        assignments: [item.assign],
      };
      sections.push(currentSection);
    } else {
      // 同一时间段，追加
      currentSection.assignments.push(item.assign);
    }
  }

  return sections;
});

// 分页状态
const currentPage = ref(1);

// 是否有上一页/下一页
const hasPrevPage = computed(() => currentPage.value > 1);
const hasNextPage = computed(() => currentPage.value < totalPages.value);

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

// 格式化角色标签（模板内直接构造，无需函数）

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
        <button
          type="button"
          class="view-toggle-btn"
          :class="{ 'view-toggle-btn--active': viewMode === 'members' }"
          @click="switchView('members')"
          :disabled="!canViewAssignments"
        >
          成员列表
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
          <span class="page-indicator">{{ currentPage }} / {{ totalPages }}</span>
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
          <div class="timeline-label" :class="`timeline-label--${section.range}`">
            {{ section.label }}
          </div>
          <div class="timeline-items">
            <div
              v-for="assign in section.assignments"
              :key="`${assign.projId}-${assign.memberId}`"
              class="assign-card"
            >
              <div class="assign-card__primary">
                <div class="assign-card__left">
                  <div class="assign-card__roles">
                    <span
                      v-if="assign.isTranslator"
                      class="assign-role-badge assign-role--translator"
                    >
                      翻译
                    </span>
                    <span
                      v-if="assign.isProofreader"
                      class="assign-role-badge assign-role--proofreader"
                    >
                      校对
                    </span>
                    <span
                      v-if="assign.isTypesetter"
                      class="assign-role-badge assign-role--typesetter"
                    >
                      嵌字
                    </span>
                    <span v-if="assign.isRedrawer" class="assign-role-badge assign-role--redrawer">
                      美工
                    </span>
                  </div>
                  <div class="assign-card__member">
                    <div v-if="assign.isPrincipal" class="assign-card__principal">
                      {{ assign.username }}
                    </div>
                    <template v-else>{{ assign.username }}</template>
                  </div>
                </div>
              </div>
              <div class="assign-card__proj">
                【{{ assign.projsetSerial }}-{{ assign.projsetIndex }}】{{ assign.projName }}
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

/* 近一天：浅蓝色 */
.timeline-label--1day {
  background: rgba(230, 245, 255, 0.95);
  color: #2f5a8f;
  border-color: rgba(118, 184, 255, 0.35);
}

/* 近一周：中等蓝色 */
.timeline-label--1week {
  background: rgba(180, 215, 255, 0.95);
  color: #1a4a7a;
  border-color: rgba(90, 160, 230, 0.5);
}

/* 近一个月：深蓝色 */
.timeline-label--1month {
  background: rgba(120, 180, 240, 0.95);
  color: #0f3a60;
  border-color: rgba(60, 130, 200, 0.6);
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
  /* keep default text direction */
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

/* Role-specific entry backgrounds */
.assign-card__roles {
  display: flex;
  gap: 6px;
  align-items: center;
}

/* Badge base */
.assign-role-badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
}

/* Role-specific small badge styles */
.assign-role--translator {
  background: rgba(237, 235, 137, 0.95); /* 淡黄色 */
  color: #836a16;
  border: 1px solid rgba(245, 220, 120, 0.6);
}

.assign-role--proofreader {
  background: rgba(255, 235, 240, 0.95); /* 粉色 */
  color: #7a2536;
  border: 1px solid rgba(255, 180, 200, 0.6);
}

.assign-role--typesetter {
  background: rgba(214, 189, 239, 0.95); /* 淡紫色 */
  color: #4a2a6a;
  border: 1px solid rgba(205, 180, 255, 0.6);
}

.assign-role--redrawer {
  background: rgba(180, 220, 160, 0.95); /* 草绿色 */
  color: #3a5a2a;
  border: 1px solid rgba(160, 200, 140, 0.6);
}

.assign-card__primary {
  display: flex;
  justify-content: flex-start;
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

.assign-card__principal {
  display: inline-block;
  font-size: 14px;
  font-weight: 600;
  color: #6b3a1a;
  padding: 2px 8px;
  border-radius: 6px;
  background: transparent;
  border: 1px solid rgba(255, 190, 140, 0.9);
}

.assign-card__time {
  font-size: 10px;
  color: #6b7c8f;
  display: flex;
  align-items: center;
  gap: 4px;
}

.assign-card__supervisor {
  font-size: 12px;
  font-weight: 600;
  color: #6b3a1a;
  padding: 4px 8px;
  border-radius: 6px;
  background: transparent;
  border: 1px solid rgba(255, 190, 140, 0.8);
  text-align: center;
}
</style>
