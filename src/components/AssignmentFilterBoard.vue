<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { FilterOption } from '../api/model/filterOption';

// 派活筛选板：仅支持按时间筛选，不需要项目集、成员等其他筛选条件
// 通过事件通知父组件，父组件负责管理真实的筛选数据

const props = defineProps<{
  disabled?: boolean;
  isSearching?: boolean;
}>();

const emit = defineEmits<{
  (e: 'add-option', option: FilterOption): void;
  (e: 'remove-option', option: FilterOption): void;
  (e: 'clear-all'): void;
}>();

// 本地预览用的筛选条件
const localFilterOptions = ref<FilterOption[]>([]);

// 控制筛选板是否启用的开关
const filterEnabled = ref<boolean>(true);

// 时间筛选状态
const selectedTimeRange = ref<'1day' | '1week' | '1month' | null>(null);

// 时间筛选：计算 time_start 并添加到筛选条件
function selectTimeRange(range: '1day' | '1week' | '1month'): void {
  console.log('[AssignmentFilterBoard] selectTimeRange called with range:', range);
  selectedTimeRange.value = range;
  const now = Math.floor(Date.now() / 1000);
  let timeStart: number;
  let label: string;

  switch (range) {
    case '1day':
      timeStart = now - 86400;
      label = '近一天';
      break;
    case '1week':
      timeStart = now - 604800;
      label = '近一周';
      break;
    case '1month':
      timeStart = now - 2592000;
      label = '近一个月';
      break;
  }

  // 先移除本地已有的时间筛选条件
  const oldTimeOption = localFilterOptions.value.find(o => o.key === 'time-start');
  if (oldTimeOption) {
    removeOption(oldTimeOption);
  }

  // 添加新的时间筛选
  const newOption: FilterOption = { label, key: 'time-start', value: String(timeStart) };
  addOption(newOption);

  console.log('[AssignmentFilterBoard] Added time filter option:', newOption);
}

function addOption(opt: FilterOption): void {
  if (props.isSearching) {
    console.log('[AssignmentFilterBoard] Cannot add option while searching');
    return;
  }

  if (!localFilterOptions.value.find(o => o.key === opt.key && o.value === opt.value)) {
    localFilterOptions.value.push(opt);
    emit('add-option', opt);
    console.log('[AssignmentFilterBoard] Option added:', opt);
  }
}

function removeOption(opt: FilterOption): void {
  console.log('[AssignmentFilterBoard] removeOption called for:', opt);

  const index = localFilterOptions.value.findIndex(o => o.key === opt.key && o.value === opt.value);

  if (index !== -1) {
    localFilterOptions.value.splice(index, 1);
    emit('remove-option', opt);
    console.log('[AssignmentFilterBoard] Option removed successfully');
  } else {
    console.log('[AssignmentFilterBoard] Option not found in local list');
  }
}

function clearAllOptions(): void {
  if (localFilterOptions.value.length === 0) {
    console.log('[AssignmentFilterBoard] No options to clear');
    return;
  }

  console.log('[AssignmentFilterBoard] Clearing all options');

  // 清空所有本地状态
  selectedTimeRange.value = null;
  localFilterOptions.value = [];

  // 通知父组件
  emit('clear-all');

  console.log('[AssignmentFilterBoard] All options cleared');
}

// 是否有选中的筛选条件
const hasFilters = computed(() => localFilterOptions.value.length > 0);

// 组件 mounted 时自动选择默认的近一天
onMounted(() => {
  console.log('[AssignmentFilterBoard] Component mounted, selecting default 1day range');
  selectTimeRange('1day');
});
</script>

<template>
  <div class="filter-board" :class="{ 'fb--disabled': !filterEnabled || props.disabled }">
    <div class="fb-header">
      <h3 class="fb-title">派活筛选</h3>
    </div>

    <!-- 时间筛选区域 -->
    <div class="fb-row fb-time-filter">
      <label class="fb-label">派遣时间</label>
      <div class="fb-time-btns">
        <button
          type="button"
          class="fb-time-btn"
          :class="{ 'fb-time-btn--active': selectedTimeRange === '1day' }"
          :disabled="props.disabled"
          @click="selectTimeRange('1day')"
        >
          近一天
        </button>
        <button
          type="button"
          class="fb-time-btn"
          :class="{ 'fb-time-btn--active': selectedTimeRange === '1week' }"
          :disabled="props.disabled"
          @click="selectTimeRange('1week')"
        >
          近一周
        </button>
        <button
          type="button"
          class="fb-time-btn"
          :class="{ 'fb-time-btn--active': selectedTimeRange === '1month' }"
          :disabled="props.disabled"
          @click="selectTimeRange('1month')"
        >
          近一个月
        </button>
      </div>
    </div>

    <!-- 筛选预览 -->
    <div v-if="hasFilters" class="fb-preview">
      <label class="fb-preview__label">当前筛选</label>
      <TransitionGroup name="chip" tag="div" class="fb-preview-chips">
        <div v-for="opt in localFilterOptions" :key="`${opt.key}-${opt.value}`" class="fb-chip">
          {{ opt.label }}
          <button
            type="button"
            class="fb-chip__remove"
            @click="removeOption(opt)"
            :disabled="props.disabled"
          >
            ×
          </button>
        </div>
      </TransitionGroup>
    </div>

    <!-- 操作按钮 -->
    <div class="fb-actions">
      <button
        type="button"
        class="fb-clear-btn"
        :disabled="!hasFilters || props.disabled"
        @click="clearAllOptions"
      >
        清空筛选
      </button>
    </div>
  </div>
</template>

<style scoped>
.filter-board {
  margin-top: 14px;
  padding: 14px 16px 8px;
  background: transparent;
  border: none;
  border-radius: 0;
  box-shadow: none;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.fb-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.fb-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #25425f;
}

.fb-row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}

.fb-label {
  display: block;
  width: 100%;
  margin: 0 0 6px 0;
  font-size: 13px;
  color: #446585;
  font-weight: 600;
}

.fb-time-filter {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.fb-time-btns {
  display: flex;
  width: 100%;
  gap: 8px;
  flex-wrap: wrap;
}

.fb-time-btn {
  flex: 1 1 0;
  min-width: 0;
  padding: 4px 4px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition:
    background 0.2s,
    border-color 0.2s,
    box-shadow 0.2s,
    transform 0.15s;
}

.fb-time-btn:hover:not(:disabled) {
  background: #eaf6ff;
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.08);
  transform: translateY(-3px);
}

.fb-time-btn--active {
  background: linear-gradient(135deg, #5ba3e0, #6db4f0);
  color: #fff;
  border-color: rgba(91, 163, 224, 0.9);
  box-shadow: 0 8px 20px rgba(91, 163, 224, 0.25);
}

.fb-time-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.fb-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.fb-preview__label {
  display: block;
  font-size: 13px;
  color: #446585;
  font-weight: 600;
}

.fb-preview-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.fb-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: linear-gradient(135deg, #e8f4ff, #f0f8ff);
  border: 1px solid rgba(118, 184, 255, 0.3);
  border-radius: 8px;
  font-size: 12px;
  color: #2a5a8c;
  font-weight: 600;
}

.fb-chip__remove {
  border: none;
  background: transparent;
  color: #5ba3e0;
  font-size: 16px;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.fb-chip__remove:hover {
  color: #ff6b6b;
}

.chip-enter-from {
  opacity: 0;
  transform: scale(0.8);
}

.chip-enter-active {
  transition: all 0.25s ease-out;
}

.chip-leave-to {
  opacity: 0;
  transform: scale(0.8);
}

.chip-leave-active {
  transition: all 0.2s ease-in;
}

.fb--disabled {
  opacity: 0.6;
}

.fb-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.fb-clear-btn {
  min-width: 80px;
  padding: 8px 16px;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  font-size: 13px;
  font-weight: 600;
  border-radius: 10px;
  cursor: pointer;
  transition:
    background 0.2s,
    box-shadow 0.2s,
    transform 0.15s;
}

.fb-clear-btn:hover:not(:disabled) {
  background: #eef6ff;
  box-shadow: 0 4px 14px rgba(118, 184, 255, 0.25);
  transform: translateY(-1px);
}

.fb-clear-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

button:disabled,
.fb-time-btn:disabled,
.fb-clear-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: rgba(246, 250, 255, 0.6);
  color: #7a8796;
}
</style>
