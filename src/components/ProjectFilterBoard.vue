<script setup lang="ts">
import { ref } from 'vue';
import RoundSwitch from './RoundSwitch.vue';
// 与示例逻辑对应的筛选面板，实现项目集/项目/成员/状态筛选
// 发出 confirmOptions 事件供父组件应用过滤

// 过滤条件项
interface FilterOption {
  label: string;
  key: string;
  value: string;
}

const filterOptions = ref<FilterOption[]>([]);

// 控制筛选板是否启用的开关（由 RoundSwitch v-model 绑定）
const filterEnabled = ref<boolean>(true);

// 输入状态
const projectSetInput = ref('');
const projectInput = ref('');
const memberInput = ref('');

// 成员下拉 mock
interface Member {
  memberId: number;
  nickname: string;
}
const showMemberDropdown = ref(false);
const filteredMemberOptions = ref<Member[]>([]);

// 状态分类选择
const selectedLabor = ref('');
const statusList = ['翻译状态', '校对状态', '嵌字状态', '监修状态', '发布状态'];

function laborToStringKey(labor: string): string {
  switch (labor) {
    case '翻译状态':
      return 'translation-status';
    case '校对状态':
      return 'proofreading-status';
    case '嵌字状态':
      return 'typesetting-status';
    case '监修状态':
      return 'supervision-status';
    case '发布状态':
      return 'release-status';
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
      return '3';
  }
}
function getOptionalValues(selectedLabor: string): string[] {
  switch (selectedLabor) {
    case '翻译状态':
    case '校对状态':
    case '嵌字状态':
    case '监修状态':
      return ['未开始', '进行中', '已完成'];
    case '发布状态':
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
function onEnterProjectSet() {
  const v = projectSetInput.value.trim();
  if (v) {
    addOption({ label: '项目集：' + v, key: 'project-set', value: v });
    projectSetInput.value = '';
  }
}
function onEnterProject() {
  const v = projectInput.value.trim();
  if (v) {
    addOption({ label: '项目：' + v, key: 'project', value: v });
    projectInput.value = '';
  }
}
function fetchMembersWithNickname(nickname: string): Promise<Member[]> {
  return new Promise(resolve => {
    setTimeout(() => {
      const mock: Member[] = [
        { memberId: 1, nickname: 'Hatsu1ki' },
        { memberId: 2, nickname: '电容' },
        { memberId: 3, nickname: '林檎' },
        { memberId: 4, nickname: '星野' },
        { memberId: 5, nickname: '白夜' },
      ];
      resolve(mock.filter(m => m.nickname.toLowerCase().includes(nickname.toLowerCase())));
    }, 300);
  });
}
async function fetchAndDisplayMembers() {
  const q = memberInput.value.trim();
  if (q) {
    const results = await fetchMembersWithNickname(q);
    filteredMemberOptions.value = results;
    showMemberDropdown.value = true;
  } else {
    showMemberDropdown.value = false;
  }
}
function selectMemberOption(member: Member) {
  addOption({ label: '成员：' + member.nickname, key: 'member', value: member.nickname });
  memberInput.value = '';
  showMemberDropdown.value = false;
  filteredMemberOptions.value = [];
}
function hideMemberDropdown() {
  setTimeout(() => (showMemberDropdown.value = false), 150);
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
  projectSetInput.value = '';
  projectInput.value = '';
  memberInput.value = '';
  selectedLabor.value = '';
}
const emit = defineEmits<{ (e: 'confirmOptions', options: FilterOption[]): void }>();
function onConfirm() {
  emit('confirmOptions', filterOptions.value);
}
</script>
<template>
  <div class="filter-board" :class="{ 'fb--disabled': !filterEnabled }">
    <div class="fb-header">
      <h3 class="fb-title">筛选控制板</h3>
      <RoundSwitch v-model="filterEnabled" />
    </div>

    <!-- 项目集输入 -->
    <div class="fb-row">
      <label class="fb-label">筛选项目集</label>
      <input
        class="fb-input"
        placeholder="输入项目集 ID 或名称 [Enter]"
        v-model="projectSetInput"
        @keyup.enter="onEnterProjectSet"
        :disabled="!filterEnabled"
      />
    </div>

    <!-- 项目输入 -->
    <div class="fb-row">
      <label class="fb-label">筛选项目</label>
      <input
        class="fb-input"
        placeholder="输入项目集号/序号/作者/标题 [Enter]"
        v-model="projectInput"
        @keyup.enter="onEnterProject"
        :disabled="!filterEnabled"
      />
    </div>

    <!-- 成员输入 -->
    <div class="fb-row fb-row--member">
      <label class="fb-label">筛选成员</label>
      <div class="fb-member-wrapper">
        <input
          class="fb-input"
          placeholder="输入成员昵称 [Enter 搜索]"
          v-model="memberInput"
          @focus="showMemberDropdown = true"
          @blur="hideMemberDropdown"
          @keyup.enter="fetchAndDisplayMembers"
          :disabled="!filterEnabled"
        />
        <ul v-if="showMemberDropdown && filteredMemberOptions.length" class="fb-dropdown">
          <li
            v-for="m in filteredMemberOptions"
            :key="m.memberId"
            class="fb-dropdown__item"
            @mousedown.prevent="selectMemberOption(m)"
          >
            {{ m.nickname }}
          </li>
        </ul>
      </div>
    </div>

    <!-- 状态选择（强制单行填充） -->
    <div class="fb-status-block">
      <label class="fb-label">筛选项目状态</label>
      <div v-if="!selectedLabor" class="fb-status-group">
        <button v-for="s in statusList" :key="s" class="fb-status-btn" @click="selectedLabor = s">
          {{ s }}
        </button>
      </div>
      <div v-else class="fb-status-group fb-status-group--detail">
        <button
          v-for="st in getOptionalValues(selectedLabor)"
          :key="st"
          class="fb-status-btn"
          @click="onSelectStatus(st)"
        >
          {{ st }}
        </button>
        <button class="fb-cancel-btn" @click="selectedLabor = ''">取消</button>
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
      <button v-if="filterOptions.length" class="fb-clear-btn" @click="clearAllOptions">
        清空条件
      </button>
      <button class="fb-confirm-btn" @click="onConfirm">确认查询</button>
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
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.fb-status-group {
  display: flex;
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
  padding: 6px 14px;
  border: none;
  background: #62a6ff;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  border-radius: 10px;
  cursor: pointer;
  box-shadow: 0 6px 16px rgba(118, 184, 255, 0.34);
}
.fb-confirm-btn:hover {
  background: #4d97fc;
}
</style>
