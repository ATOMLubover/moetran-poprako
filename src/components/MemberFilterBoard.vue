<script setup lang="ts">
import { ref, computed } from 'vue';
import type { FilterOption } from '../api/model/filterOption';

// 成员筛选板：支持按职位和名字模糊搜索
// 通过事件通知父组件，父组件负责管理真实的筛选数据

const props = defineProps<{
  teamId?: string;
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

// 职位选择
const selectedPositionKey = ref<string>('');
// 职位列表（互斥按钮）
const positionList = [
  { label: '翻译', key: 'translator' },
  { label: '校对', key: 'proofreader' },
  { label: '嵌字', key: 'typesetter' },
  { label: '美工', key: 'redrawer' },
  { label: '监修', key: 'principal' },
];

// 名字模糊搜索输入
const memberNameInput = ref('');

function togglePosition(position: { label: string; key: string }): void {
  if (!teamAvailable.value || props.disabled) return;

  const current = localFilterOptions.value.find(opt => opt.key === 'position');

  if (selectedPositionKey.value === position.key) {
    selectedPositionKey.value = '';
    if (current) {
      removeOption(current);
    }
    return;
  }

  if (current) {
    removeOption(current);
  }

  selectedPositionKey.value = position.key;
  const option: FilterOption = {
    label: `职位：${position.label}`,
    key: 'position',
    value: position.key,
  };
  addOption(option);
}

// 添加名字筛选
function onEnterMemberName(): void {
  const v = memberNameInput.value.trim();
  if (v) {
    addOption({ label: `成员：${v}`, key: 'fuzzy-name', value: v });
    memberNameInput.value = '';
  }
}

function addOption(opt: FilterOption): void {
  if (props.isSearching) {
    console.log('[MemberFilterBoard] Cannot add option while searching');
    return;
  }

  if (!localFilterOptions.value.find(o => o.key === opt.key && o.value === opt.value)) {
    localFilterOptions.value.push(opt);
    emit('add-option', opt);
    console.log('[MemberFilterBoard] Option added:', opt);
  }
}

function removeOption(opt: FilterOption): void {
  console.log('[MemberFilterBoard] removeOption called for:', opt);

  const index = localFilterOptions.value.findIndex(o => o.key === opt.key && o.value === opt.value);

  if (index !== -1) {
    localFilterOptions.value.splice(index, 1);
    emit('remove-option', opt);
    console.log('[MemberFilterBoard] Option removed successfully');
  } else {
    console.log('[MemberFilterBoard] Option not found in local list');
  }
}

function clearAllOptions(): void {
  if (localFilterOptions.value.length === 0) {
    console.log('[MemberFilterBoard] No options to clear');
    return;
  }

  console.log('[MemberFilterBoard] Clearing all options');

  // 清空所有本地状态
  memberNameInput.value = '';
  selectedPositionKey.value = '';
  localFilterOptions.value = [];

  // 通知父组件
  emit('clear-all');

  console.log('[MemberFilterBoard] All options cleared');
}

// 是否有选中的筛选条件
const hasFilters = computed(() => localFilterOptions.value.length > 0);

// 当前是否有 teamId（当无 teamId 时筛选控件应被禁用）
const teamAvailable = computed(() => !!props.teamId);
</script>

<template>
  <div class="filter-board" :class="{ 'fb--disabled': !filterEnabled || props.disabled }">
    <div class="fb-header">
      <h3 class="fb-title">成员筛选</h3>
    </div>

    <!-- 职位筛选区域 -->
    <div class="fb-row">
      <label class="fb-label">按职位筛选</label>
      <div class="fb-position-group">
        <button
          v-for="pos in positionList"
          :key="pos.key"
          type="button"
          class="fb-position-btn"
          :class="{ 'fb-position-btn--active': selectedPositionKey === pos.key }"
          :disabled="!teamAvailable || props.disabled"
          @click="togglePosition(pos)"
        >
          {{ pos.label }}
        </button>
      </div>
    </div>

    <!-- 名字模糊搜索 -->
    <div class="fb-row">
      <label class="fb-label">成员名字</label>
      <input
        v-model="memberNameInput"
        type="text"
        class="fb-input"
        placeholder="输入成员名字（模糊搜索）"
        :disabled="!teamAvailable || props.disabled"
        @keyup.enter="onEnterMemberName"
      />
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
  width: 100%;
}

.fb-row > .fb-input,
.fb-row > .fb-position-group {
  width: 100%;
  box-sizing: border-box;
}

.fb-label {
  display: block;
  width: 100%;
  margin: 0 0 6px 0;
  font-size: 13px;
  color: #446585;
  font-weight: 600;
}

.fb-input {
  flex: 1;
  width: 100%;
  height: 40px;
  border: 1px solid rgba(170, 200, 232, 0.8);
  border-radius: 10px;
  padding: 8px 12px;
  font-size: 13px;
  background: rgba(248, 252, 255, 0.95);
  color: #203a56;
  box-sizing: border-box;
}

.fb-select {
  flex: 1;
  width: 100%;
  height: 34px;
  border: 1px solid rgba(170, 200, 232, 0.8);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 13px;
  background: rgba(248, 252, 255, 0.95);
  color: #203a56;
  cursor: pointer;
  outline: none;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.fb-select:hover:not(:disabled) {
  border-color: rgba(118, 184, 255, 0.9);
}

.fb-select:focus {
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 0 0 1px rgba(118, 184, 255, 0.55);
}

.fb-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.fb-position-group {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  width: 100%;
}

.fb-position-btn {
  flex: 1 1 0;
  min-width: 0;
  padding: 6px 10px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  border-radius: 8px;
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 0.18s ease,
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.fb-position-btn:hover:not(:disabled) {
  background: #eaf6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.1);
  transform: translateY(-2px);
}

.fb-position-btn--active {
  background: rgba(210, 244, 225, 0.96);
  color: #1e6042;
  border-color: rgba(140, 205, 170, 0.9);
  box-shadow: 0 6px 18px rgba(120, 200, 160, 0.08);
}

.fb-position-btn:disabled {
  opacity: 0.55;
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
.fb-clear-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: rgba(246, 250, 255, 0.6);
  color: #7a8796;
}
</style>
