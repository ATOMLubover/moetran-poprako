<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import MemberSelector, { MemberInfo } from './MemberSelector.vue';
import { getTeamPoprakoProjsets, type PoprakoProjsetInfo } from '../ipc/project';
import { useToastStore } from '../stores/toast';
// 与示例逻辑对应的筛选面板，实现项目集/项目/成员/状态筛选
// 发出 confirmOptions 事件供父组件应用过滤

// 父组件传入的上下文：当前选中的团队 ID（用于成员搜索）
const props = defineProps<{
  teamId?: string;
  modelValue: FilterOption[]; // v-model 绑定的筛选条件
  disabled?: boolean; // 是否禁用整个筛选面板
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', options: FilterOption[]): void;
  (e: 'applyFilter'): void; // 通知父组件应用筛选（确认查询或清空时触发）
  (e: 'openProjsetCreator'): void; // 通知父组件打开项目集创建弹窗
}>();

const toastStore = useToastStore();

// 过滤条件项
export interface FilterOption {
  label: string;
  key: string;
  value: string;
}

// 使用 computed 访问父组件传入的 filterOptions
const filterOptions = computed({
  get: () => props.modelValue,
  set: (val: FilterOption[]) => emit('update:modelValue', val),
});

// 高级成员选择（职位+多选）
const advancedPickedMembers = ref<MemberInfo[]>([]);
const memberSelectorOpen = ref(false);

function openMemberSelector(): void {
  memberSelectorOpen.value = true;
}

function handleMemberSelectorConfirm(): void {
  const labelMap: Record<MemberInfo['position'], string> = {
    translator: '翻译',
    proofreader: '校对',
    typesetter: '嵌字',
    redrawer: '美工',
    principal: '监修',
  };
  for (const m of advancedPickedMembers.value) {
    const key = `member-${m.position}`;
    const label = `成员(${labelMap[m.position]})：${m.name}`;
    if (!filterOptions.value.find(o => o.key === key && o.value === m.id)) {
      filterOptions.value.push({ label, key, value: m.id });
    }
  }
  advancedPickedMembers.value = [];
}

function handleMemberSelectorCancel(): void {
  // 取消不做合并
}

function handleMemberSelectorClose(): void {
  memberSelectorOpen.value = false;
}

// 控制筛选板是否启用的开关
const filterEnabled = ref<boolean>(true);

// 当前是否有 teamId（当无 teamId 时，除“筛选项目”外其余控件应被禁用）
const teamAvailable = computed(() => !!props.teamId);

// 是否可以执行确认查询：存在筛选项且面板启用，且 (有 team 或 包含 project 条件)
const canConfirm = computed(() => {
  if (!filterEnabled.value) return false;
  if (!filterOptions.value.length) return false;
  if (teamAvailable.value) return true;
  return filterOptions.value.some(o => o.key === 'project');
});

// 项目集列表与加载状态
const currentProjsets = ref<PoprakoProjsetInfo[]>([]);
const projsetLoading = ref(false);
// projset 选择弹窗状态与临时选中项
const projsetSelectorOpen = ref(false);
const projsetPickedIds = ref<string[]>([]);
// 时间筛选状态（time_start unix 时间戳）
const selectedTimeRange = ref<'1day' | '1week' | '1month' | null>(null);

// 监听 teamId 变化，重置并重新加载项目集列表
watch(
  () => props.teamId,
  () => {
    // reset state when team changes; if no teamId provided, close selector and clear lists
    if (!props.teamId) {
      currentProjsets.value = [];
      projsetLoading.value = false;
      projsetPickedIds.value = [];
      projsetSelectorOpen.value = false;
      return;
    }

    currentProjsets.value = [];
    projsetLoading.value = false;
    projsetPickedIds.value = [];
    void loadProjsetsForCurrentTeam();
  },
  { immediate: true }
);

// 加载当前团队的项目集列表
async function loadProjsetsForCurrentTeam(): Promise<void> {
  const teamId = props.teamId ?? '';
  if (!teamId || projsetLoading.value) return;

  projsetLoading.value = true;
  try {
    const list = await getTeamPoprakoProjsets(teamId);
    console.log('Loaded projsets for filter board', teamId, list);
    currentProjsets.value = list;
  } catch (e) {
    console.error('Failed to load projsets for filter board', teamId, e);
    toastStore.show('加载项目集失败，请稍后重试');
  } finally {
    projsetLoading.value = false;
  }
}

// 输入状态
const projectInput = ref('');
const memberInput = ref('');

// 状态分类选择
const selectedLabor = ref('');
// Labels simplified per request
const statusList = ['翻译', '校对', '嵌字', '监修', '发布'];

function laborToStringKey(labor: string): string {
  switch (labor) {
    case '翻译':
      return 'translation-status';
    case '校对':
      return 'proofreading-status';
    case '嵌字':
      return 'typesetting-status';
    case '监修':
      return 'reviewing-status';
    case '发布':
      return 'publish-status';
    default:
      return '';
  }
}
function statusTypeToStringValue(statusType: string): string {
  switch (statusType) {
    case '未开始':
      return '0';
    case '进行中':
      return '1';
    case '已完成':
      return '2';
    default:
      return '';
  }
}
function getOptionalValues(selectedLabor: string): string[] {
  switch (selectedLabor) {
    case '翻译':
    case '校对':
    case '嵌字':
    case '监修':
      return ['未开始', '进行中', '已完成'];
    case '发布':
      return ['未开始', '已完成'];
    default:
      return [];
  }
}
function onSelectStatus(statusType: string) {
  const optionLabel = selectedLabor.value + '：' + statusType;
  const existing = filterOptions.value.find(o => o.label === optionLabel);
  if (!existing) {
    addOption({
      label: optionLabel,
      key: laborToStringKey(selectedLabor.value),
      value: statusTypeToStringValue(statusType),
    });
  }
  selectedLabor.value = '';
}
function openProjsetSelector(): void {
  // only open when a teamId is available
  if (!props.teamId) return;

  // prefill picked ids with existing project-set options
  projsetPickedIds.value = filterOptions.value
    .filter(o => o.key === 'project-set')
    .map(o => o.value);
  projsetSelectorOpen.value = true;
}

// 重新加载项目集列表的公开方法（供父组件在创建项目集后调用）
function reloadProjsets(): void {
  void loadProjsetsForCurrentTeam();
}

// 暴露给父组件
defineExpose({
  reloadProjsets,
});

// 时间筛选：计算 time_start 并添加到筛选条件
function selectTimeRange(range: '1day' | '1week' | '1month'): void {
  selectedTimeRange.value = range;
  const now = Math.floor(Date.now() / 1000);
  let timeStart: number;
  let label: string;

  switch (range) {
    case '1day':
      timeStart = now - 86400;
      label = '时间：近一天';
      break;
    case '1week':
      timeStart = now - 604800;
      label = '时间：近一周';
      break;
    case '1month':
      timeStart = now - 2592000;
      label = '时间：近一个月';
      break;
  }

  // 移除已有的时间筛选条件，添加新的
  filterOptions.value = filterOptions.value.filter(o => o.key !== 'time-start');
  addOption({ label, key: 'time-start', value: String(timeStart) });
}

function onProjsetConfirm(): void {
  for (const id of projsetPickedIds.value) {
    const projset = currentProjsets.value.find(p => p.projsetId === id);
    if (!projset) continue;
    const label = `项目集：${projset.projsetName}`;
    addOption({ label, key: 'project-set', value: id });
  }
  projsetSelectorOpen.value = false;
}

function onProjsetCancel(): void {
  projsetSelectorOpen.value = false;
}

function onEnterProject() {
  const v = projectInput.value.trim();
  if (v) {
    addOption({ label: '项目：' + v, key: 'project', value: v });
    projectInput.value = '';
  }
}
function addOption(opt: FilterOption) {
  if (!filterOptions.value.find(o => o.key === opt.key && o.value === opt.value)) {
    filterOptions.value.push(opt);
  }
}
function removeOption(opt: FilterOption) {
  filterOptions.value = filterOptions.value.filter(
    o => !(o.key === opt.key && o.value === opt.value)
  );
}
function clearAllOptions() {
  filterOptions.value = [];
  projectInput.value = '';
  memberInput.value = '';
  selectedLabor.value = '';
  projsetPickedIds.value = [];
  advancedPickedMembers.value = [];
  // 清空时立即通知父组件应用空筛选（触发列表刷新）
  emit('applyFilter');
}

function onConfirm() {
  // protect: ignore confirm when no conditions
  if (!filterOptions.value.length) {
    toastStore.show('请先添加筛选条件');
    return;
  }

  // diagnostic log: print the options being emitted so we can debug unexpected member_ids
  // (will be visible in renderer devtools console)
  // eslint-disable-next-line no-console
  console.log(
    '[ProjectFilterBoard] confirmOptions ->',
    JSON.parse(JSON.stringify(filterOptions.value))
  );
  // 通知父组件应用当前筛选条件
  emit('applyFilter');
}
</script>
<template>
  <div class="filter-board" :class="{ 'fb--disabled': !filterEnabled || props.disabled }">
    <div class="fb-header">
      <h3 class="fb-title">PopRaKo 筛选控制板</h3>
      <button class="fb-confirm-btn" @click="onConfirm" :disabled="!canConfirm">确认查询</button>
    </div>

    <!-- 第一行：项目名（模糊搜索）+ 项目集选择（右对齐） -->
    <div class="fb-row fb-row--tight" style="align-items: center">
      <label class="fb-label">项目名称</label>
      <input
        class="fb-input"
        placeholder="输入项目集号/序号/作者/标题 [Enter 确认]"
        v-model="projectInput"
        @keyup.enter="onEnterProject"
        :disabled="!filterEnabled"
      />
      <div style="display: flex; align-items: center; gap: 8px; margin-left: auto">
        <label class="fb-label fb-label--small">项目集</label>
        <button
          class="fb-adv-btn"
          @click="openProjsetSelector"
          :disabled="!filterEnabled || !props.teamId || projsetLoading"
        >
          选择
        </button>
      </div>
    </div>

    <!-- 第三行：状态选择 -->
    <div class="fb-status-block fb-row--tight">
      <label class="fb-label">项目状态</label>
      <div v-if="!selectedLabor" class="fb-status-group">
        <button
          v-for="s in statusList"
          :key="s"
          class="fb-status-btn"
          @click="selectedLabor = s"
          :disabled="!filterEnabled || !teamAvailable"
        >
          {{ s }}
        </button>
      </div>
      <div v-else class="fb-status-group fb-status-group--detail">
        <button
          v-for="st in getOptionalValues(selectedLabor)"
          :key="st"
          class="fb-status-btn"
          @click="onSelectStatus(st)"
          :disabled="!filterEnabled || !teamAvailable"
        >
          {{ st }}
        </button>
        <button
          class="fb-cancel-btn"
          @click="selectedLabor = ''"
          :disabled="!filterEnabled || !teamAvailable"
        >
          取消
        </button>
      </div>
    </div>

    <!-- 状态选择（强制单行填充） -->

    <!-- 时间筛选 + 成员筛选 -->
    <div class="fb-row fb-row--tight fb-time-filter" style="align-items: center">
      <label class="fb-label">发布时间</label>
      <div class="fb-time-btns" style="flex: 1; display: flex; gap: 8px">
        <button
          class="fb-time-btn"
          :class="{ 'fb-time-btn--active': selectedTimeRange === '1day' }"
          @click="selectTimeRange('1day')"
          :disabled="!filterEnabled || !teamAvailable"
        >
          近一天
        </button>
        <button
          class="fb-time-btn"
          :class="{ 'fb-time-btn--active': selectedTimeRange === '1week' }"
          @click="selectTimeRange('1week')"
          :disabled="!filterEnabled || !teamAvailable"
        >
          近一周
        </button>
        <button
          class="fb-time-btn"
          :class="{ 'fb-time-btn--active': selectedTimeRange === '1month' }"
          @click="selectTimeRange('1month')"
          :disabled="!filterEnabled || !teamAvailable"
        >
          近一个月
        </button>
      </div>
      <div class="fb-member-compact">
        <label class="fb-label fb-label--small">成员</label>
        <button
          type="button"
          class="fb-adv-btn"
          @click="openMemberSelector"
          :disabled="!filterEnabled || !teamAvailable"
        >
          选择
        </button>
      </div>
    </div>

    <!-- 条件预览：使用 transition-group 动画展示 -->
    <div class="fb-preview" v-if="filterOptions.length">
      <span class="fb-preview__label">筛选条件预览：</span>
      <transition-group name="chip" tag="div" class="fb-preview-chips">
        <div v-for="opt in filterOptions" :key="opt.key + opt.value" class="fb-chip">
          {{ opt.label }}
          <button class="fb-chip__remove" @click="removeOption(opt)">&times;</button>
        </div>
      </transition-group>
    </div>

    <!-- 操作按钮 -->
    <div class="fb-actions">
      <button
        v-if="filterOptions.length"
        class="fb-clear-btn"
        @click="clearAllOptions"
        :disabled="!filterEnabled"
      >
        清空条件
      </button>
    </div>
    <MemberSelector
      :show="memberSelectorOpen"
      :picked="advancedPickedMembers"
      :team-id="props.teamId"
      @confirm="handleMemberSelectorConfirm"
      @cancel="handleMemberSelectorCancel"
      @close="handleMemberSelectorClose"
    />

    <!-- 项目集选择弹窗 -->
    <div v-if="projsetSelectorOpen" class="projset-overlay">
      <div class="projset-panel">
        <div class="selector-header">
          <div class="selector-title">选择项目集</div>
          <button class="selector-close" @click="onProjsetCancel">✕</button>
        </div>
        <div class="selector-body">
          <div style="max-height: 280px; overflow: auto">
            <label v-for="ps in currentProjsets" :key="ps.projsetId" class="projset-item">
              <input type="checkbox" :value="ps.projsetId" v-model="projsetPickedIds" />
              <span style="margin-left: 8px">{{ ps.projsetName }}</span>
            </label>
            <div v-if="!currentProjsets.length" class="selector-empty">暂无项目集</div>
          </div>
          <div class="selector-actions">
            <button
              class="selector-action-btn selector-action-btn--cancel"
              @click="onProjsetCancel"
            >
              取消
            </button>
            <button
              class="selector-action-btn selector-action-btn--confirm"
              @click="onProjsetConfirm"
            >
              添加所选
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped>
.filter-board {
  margin-top: 14px;
  padding: 14px 16px 18px;
  background: rgba(250, 253, 255, 0.92);
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 14px;
  box-shadow: 0 8px 22px rgba(140, 180, 230, 0.14);
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
  align-items: center;
  gap: 12px;
}
.fb-row--member {
  position: relative;
}
.fb-label {
  min-width: 92px;
  font-size: 13px;
  color: #446585;
  font-weight: 600;
}
.fb-input {
  flex: 1;
  height: 34px;
  border: 1px solid rgba(170, 200, 232, 0.8);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 13px;
  background: rgba(248, 252, 255, 0.95);
  color: #203a56;
}
.fb-projset-wrapper {
  flex: 1;
  display: flex;
}
.fb-select {
  flex: 1;
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
    border-color 0.14s ease,
    box-shadow 0.14s ease;
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
.fb-member-wrapper {
  flex: 1;
  position: relative;
}
.fb-dropdown {
  position: absolute;
  top: 36px;
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid rgba(170, 200, 232, 0.8);
  border-radius: 10px;
  box-shadow: 0 10px 24px rgba(140, 180, 230, 0.18);
  max-height: 180px;
  overflow: auto;
  padding: 4px 0;
  list-style: none;
  margin: 0;
}
.fb-dropdown__item {
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
}
.fb-dropdown__item:hover {
  background: #f1f7ff;
}
.fb-status-block {
  /* lay out label and status buttons in one row, matching other fb-row
     controls: label on the left, buttons fill the remaining space */
  display: flex;
  align-items: center;
  gap: 12px;
}
.fb-status-group {
  display: flex;
  flex: 1 1 auto; /* fill remaining horizontal space beside the label */
  flex-wrap: nowrap; /* 强制单行 */
  gap: 8px;
}
.fb-status-btn {
  flex: 1 1 0; /* 横向填充并允许缩放，不换行 */
  min-width: 0;
  padding: 8px 10px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition:
    transform 0.22s cubic-bezier(0.2, 0.9, 0.2, 1),
    background 0.18s ease,
    box-shadow 0.18s ease;
}
.fb-status-btn:hover {
  background: #eaf6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.08);
  transform: translateY(-3px);
}
.fb-status-group--detail {
  align-items: center;
}
.fb-cancel-btn {
  padding: 6px 10px;
  border: none;
  background: #ffb3b3;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
}
.fb-cancel-btn:hover {
  background: #ff9b9b;
}
.fb-preview {
  display: flex;
  align-items: center;
  gap: 12px;
}
.fb-preview__label {
  font-size: 13px;
  font-weight: 500;
}
.fb-preview-chips {
  display: flex;
  gap: 8px;
  align-items: center;
}
.fb-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: #f0fbf3;
  color: #21593a;
  font-size: 13px;
  border: 1px solid #bfe6c6;
  border-radius: 999px;
  box-shadow: 0 6px 18px rgba(120, 200, 160, 0.06);
  transform-origin: center;
}
.fb-chip__remove {
  border: none;
  background: transparent;
  color: #c24b4b;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}
.fb-chip__remove:hover {
  color: #8f1212;
}

/* transition-group 动画 */
.chip-enter-from {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}
.chip-enter-active {
  transition: all 220ms cubic-bezier(0.2, 0.9, 0.2, 1);
}
.chip-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}
.chip-leave-active {
  transition: all 180ms ease;
}

/* 面板禁用态 */
.fb--disabled {
  opacity: 0.6;
}
.fb--disabled .fb-input {
  background: #f3f6f9;
}
.fb-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 4px;
}
.fb-member-role-btns {
  display: inline-flex;
  gap: 8px;
}
.fb-role-btn {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  cursor: pointer;
}
.fb-role-btn:hover {
  background: #eaf6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.08);
  transform: translateY(-2px);
}
.fb-clear-btn {
  padding: 6px 14px;
  border: none;
  background: #ffb3b3;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  border-radius: 10px;
  cursor: pointer;
}
.fb-clear-btn:hover {
  background: #ff9b9b;
}
.fb-confirm-btn {
  padding: 6px 10px;
  border-radius: 10px; /* 方正却圆滑 */
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff; /* 轻色背景，去掉渐变 */
  color: #2f5a8f;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.06);
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    background 0.12s ease;
}
.fb-confirm-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.12);
  background: #eef6ff;
}
.fb-confirm-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
}
.fb-adv-btn {
  /* Match role button styling for consistent visual alignment */
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  font-weight: 400;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    background 0.12s ease,
    box-shadow 0.12s ease;
  min-width: 138px; /* Ensure member/projset buttons keep a readable width */
}
.fb-adv-btn:hover {
  background: #eaf6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.08);
  transform: translateY(-2px);
}
.fb-adv-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.fb-adv-disabled-note {
  font-size: 12px;
  color: #7a8b99;
  margin-left: 8px;
}
.fb-adv-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  font-size: 12px;
  color: #2f4b66;
}
.fb-adv-chip {
  background: #f0fbf3;
  border: 1px solid #bfe6c6;
  padding: 3px 8px;
  border-radius: 999px;
  display: inline-flex;
  gap: 4px;
  align-items: center;
}
.fb-adv-role {
  color: #3a6f4d;
  font-weight: 600;
}
/* 时间筛选按钮样式 */
.fb-time-filter {
  display: flex;
  align-items: center;
  gap: 12px;
}
.fb-time-btns {
  display: flex;
  gap: 8px;
  flex: 1 1 auto; /* 占据发布时间标签右侧的剩余宽度 */
  min-width: 0; /* 允许子项在窄容器中正确收缩 */
}
.fb-time-btn {
  padding: 6px 10px; /* 缩减高度 */
  height: 30px;
  line-height: 1;
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
  flex: 1 1 0; /* 三个按钮等分父容器宽度 */
  min-width: 0; /* 避免溢出导致布局被破坏 */
}

.fb-member-compact {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto; /* 右对齐到整行末端，参考项目集一行的实现 */
  flex: 0 0 auto;
  min-width: 140px; /* 在窄屏下仍保留基本空间 */
  justify-content: flex-end; /* 内容靠右，使按钮贴近右侧边缘 */
}
.fb-time-btn:hover:not(:disabled) {
  background: #eaf6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.08);
  transform: translateY(-2px);
}
.fb-time-btn--active {
  background: linear-gradient(135deg, #5ba3e0, #6db4f0);
  color: #fff;
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 4px 12px rgba(118, 184, 255, 0.25);
}
.fb-time-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
/* 项目集弹窗样式 */
.projset-overlay {
  position: fixed;
  inset: 0;
  background: rgba(10, 20, 40, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}
.projset-creator-modal {
  width: 600px;
  max-width: calc(100% - 40px);
  max-height: calc(100vh - 80px);
  overflow: hidden;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(30, 60, 100, 0.4);
}
.projset-panel {
  width: 420px;
  max-width: calc(100% - 40px);
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 20px 48px rgba(30, 60, 100, 0.35);
  padding: 12px 14px;
  box-sizing: border-box;
}
.projset-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 6px;
}
.selector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.selector-title {
  font-weight: 700;
  color: #203650;
}
.selector-close {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
}
.selector-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 10px;
}
.selector-action-btn {
  min-width: 86px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
}
.selector-action-btn--cancel {
  background: #f3f5f9;
  border: none;
  color: #4a5f7a;
}
.selector-action-btn--confirm {
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #fff;
  border: none;
}
.selector-empty {
  text-align: center;
  color: #6b7c91;
  padding: 12px 0;
}
.fb-row--tight {
  gap: 8px;
}
.fb-row-right {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}
.fb-label--small {
  min-width: 0;
  width: auto;
}

/* Disabled state styling for interactive buttons */
button:disabled,
.fb-role-btn:disabled,
.fb-status-btn:disabled,
.fb-cancel-btn:disabled,
.fb-clear-btn:disabled,
.fb-confirm-btn:disabled,
.fb-adv-btn:disabled,
.selector-action-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  pointer-events: none;
  filter: grayscale(20%);
}

/* Make disabled role/status buttons visually consistent */
.fb-role-btn:disabled,
.fb-status-btn:disabled {
  background: #f3f5f9;
  color: #9aa7b8;
  border: 1px solid rgba(150, 180, 210, 0.2);
}

/* Ensure cancel/confirm/clear show disabled look when disabled */
.fb-cancel-btn:disabled {
  background: #f0f0f0;
  color: #9aa7b8;
}
.fb-confirm-btn:disabled {
  background: linear-gradient(135deg, #b7d8ff, #9abff0);
  color: #ffffff;
}
</style>
