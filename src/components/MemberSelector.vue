<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import type { MemberPosition } from '../ipc/member';
import { ResMemberBrief } from '../api/model/member';

// 通用成员类型（包含职位）
export interface MemberInfo {
  id: string;
  name: string;
  position: 'translator' | 'proofreader' | 'typesetter' | 'redrawer' | 'principal';
}

// Props 设计：
// picked: 外部传入的数组（父组件基于 ref 传递，可被本组件直接 push / splice）
// show: 是否显示选择器（由父控制）
// fetchMembers: 可选的自定义拉取函数 (position, keyword) => Promise<MemberInfo[]>
// teamId: 可选的团队 ID（真实实现时传给后端）
const props = defineProps<{
  picked: MemberInfo[];
  show: boolean;
  fetchMembers?: (
    position: MemberInfo['position'],
    keyword: string,
    teamId?: string
  ) => Promise<MemberInfo[]>;
  teamId?: string;
  // optional initial role to open on (translator|proofreader|typesetter)
  initialRole?: MemberInfo['position'];
}>();

const emit = defineEmits<{ (e: 'close'): void; (e: 'confirm'): void; (e: 'cancel'): void }>();

// 内部状态
const currentRole = ref<MemberInfo['position']>('translator');
const keyword = ref('');
const loading = ref(false);
const results = ref<MemberInfo[]>([]);
const snapshot = ref<MemberInfo[]>([]); // 打开时的快照用于取消回滚

// 中文职位标签映射
const ROLE_LABEL: Record<MemberInfo['position'], string> = {
  translator: '翻译',
  proofreader: '校对',
  typesetter: '嵌字',
  redrawer: '美工',
  principal: '监修',
};

// 默认实现：通过 IPC 调用后端成员搜索接口
async function defaultFetchMembers(
  role: MemberInfo['position'],
  kw: string,
  teamId?: string
): Promise<MemberInfo[]> {
  if (!teamId) {
    return [];
  }

  const params = {
    teamId: teamId,
    position: role as MemberPosition,
    fuzzyName: kw.trim() || undefined,
    page: 1,
    limit: 20,
  } satisfies {
    teamId: string;
    position: MemberPosition;
    fuzzyName?: string;
    page?: number;
    limit?: number;
  };

  const { searchMembersByName } = await import('../ipc/member');
  const list = await searchMembersByName(params);
  return list.map((m: ResMemberBrief) => ({
    id: m.memberId,
    name: m.username,
    position: role,
  }));
}

// 初始化或显示时建立快照 & 初始展示已选
watch(
  () => props.show,
  show => {
    if (show) {
      snapshot.value = props.picked.map(m => ({ ...m }));
      // 初始展示：当前角色已选成员
      // 如果父组件希望初始打开到某个职位，使用 initialRole
      if (props.initialRole) {
        currentRole.value = props.initialRole;
      }
      refreshResults();
    } else {
      keyword.value = '';
      results.value = [];
      loading.value = false;
    }
  }
);

async function refreshResults() {
  if (!props.show) return;
  loading.value = true;
  try {
    const fetcher = props.fetchMembers || defaultFetchMembers;
    const fetched = await fetcher(currentRole.value, keyword.value.trim(), props.teamId);
    const pickedIds = new Set(
      props.picked.filter(m => m.position === currentRole.value).map(m => m.id)
    );
    // 已选成员放在顶部，其余按搜索结果顺序
    const pickedMembers = props.picked.filter(m => m.position === currentRole.value);
    const extra = fetched.filter(m => !pickedIds.has(m.id));
    results.value = [...pickedMembers, ...extra];
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  refreshResults();
}

function switchRole(role: MemberInfo['position']) {
  currentRole.value = role;
  refreshResults();
}

function isPicked(member: MemberInfo): boolean {
  return props.picked.some(m => m.id === member.id && m.position === member.position);
}

function addMember(member: MemberInfo) {
  if (!isPicked(member)) {
    props.picked.push({ id: member.id, name: member.name, position: member.position });
  }
  if (!keyword.value.trim()) {
    // 无关键词时即时刷新，以保证顶部显示
    refreshResults();
  }
}

function removeMember(member: MemberInfo) {
  const idx = props.picked.findIndex(m => m.id === member.id && m.position === member.position);
  if (idx !== -1) props.picked.splice(idx, 1);
  if (!keyword.value.trim()) {
    refreshResults();
  }
}

function confirm() {
  emit('confirm');
  emit('close');
}
function cancel() {
  // 回滚到快照
  props.picked.splice(0, props.picked.length, ...snapshot.value.map(m => ({ ...m })));
  emit('cancel');
  emit('close');
}
function close() {
  emit('close');
}

const pickedDisplayByRole = computed(() => {
  const groups: Record<MemberInfo['position'], string[]> = {
    translator: [],
    proofreader: [],
    typesetter: [],
    redrawer: [],
    principal: [],
  };
  for (const m of props.picked) {
    groups[m.position].push(m.name);
  }
  return groups;
});
</script>

<template>
  <div v-if="show" class="ms-overlay">
    <div class="ms-panel">
      <header class="ms-header">
        <nav class="ms-roles">
          <button
            v-for="r in ['translator', 'proofreader', 'typesetter', 'redrawer', 'principal']"
            :key="r"
            type="button"
            class="ms-role-btn"
            :class="{ 'ms-role-btn--active': currentRole === r }"
            @click="switchRole(r as MemberInfo['position'])"
          >
            {{ ROLE_LABEL[r as MemberInfo['position']] }}
          </button>
        </nav>
        <button type="button" class="ms-close" @click="close" title="关闭">×</button>
      </header>
      <div class="ms-body">
        <div class="ms-search">
          <input
            v-model="keyword"
            type="text"
            class="ms-input"
            placeholder="输入成员名称搜索"
            @keyup.enter="handleSearch"
          />
          <button type="button" class="ms-search-btn" @click="handleSearch" :disabled="loading">
            {{ loading ? '搜索中...' : '搜索' }}
          </button>
        </div>
        <ul class="ms-list" v-if="results.length">
          <li
            v-for="m in results"
            :key="m.id + m.position"
            class="ms-item"
            :class="{ 'ms-item--picked': isPicked(m) }"
          >
            <span class="ms-item__name">{{ m.name }}</span>
            <span class="ms-item__role">{{ ROLE_LABEL[m.position] }}</span>
            <button
              v-if="isPicked(m)"
              type="button"
              class="ms-icon-btn ms-icon-btn--remove"
              @click.stop="removeMember(m)"
              title="移除该成员"
            >
              −
            </button>
            <button
              v-else
              type="button"
              class="ms-icon-btn ms-icon-btn--add"
              @click.stop="addMember(m)"
              title="加入该成员"
            >
              +
            </button>
          </li>
        </ul>
        <div v-else class="ms-empty">暂无结果</div>

        <div class="ms-picked" v-if="props.picked.length">
          <div class="ms-picked-row" v-if="pickedDisplayByRole.translator.length">
            <span class="ms-picked-label">翻译：</span>
            <span class="ms-picked-names">{{ pickedDisplayByRole.translator.join('、') }}</span>
          </div>
          <div class="ms-picked-row" v-if="pickedDisplayByRole.proofreader.length">
            <span class="ms-picked-label">校对：</span>
            <span class="ms-picked-names">{{ pickedDisplayByRole.proofreader.join('、') }}</span>
          </div>
          <div class="ms-picked-row" v-if="pickedDisplayByRole.typesetter.length">
            <span class="ms-picked-label">嵌字：</span>
            <span class="ms-picked-names">{{ pickedDisplayByRole.typesetter.join('、') }}</span>
          </div>
          <div class="ms-picked-row" v-if="pickedDisplayByRole.redrawer.length">
            <span class="ms-picked-label">美工：</span>
            <span class="ms-picked-names">{{ pickedDisplayByRole.redrawer.join('、') }}</span>
          </div>
          <div class="ms-picked-row" v-if="pickedDisplayByRole.principal.length">
            <span class="ms-picked-label">监修：</span>
            <span class="ms-picked-names">{{ pickedDisplayByRole.principal.join('、') }}</span>
          </div>
        </div>

        <div class="ms-actions">
          <button type="button" class="ms-action ms-action--cancel" @click="cancel">取消</button>
          <button type="button" class="ms-action ms-action--confirm" @click="confirm">确认</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ms-overlay {
  position: fixed;
  inset: 0;
  background: rgba(20, 40, 70, 0.28);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.ms-panel {
  width: 400px;
  max-width: calc(100% - 40px);
  background: #fff;
  border-radius: 18px;
  box-shadow: 0 20px 48px rgba(40, 70, 120, 0.5);
  padding: 16px 18px 18px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ms-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.ms-roles {
  display: flex;
  gap: 8px;
}
.ms-role-btn {
  padding: 6px 14px;
  border-radius: 999px;
  border: 1px solid rgba(170, 190, 215, 0.85);
  background: #f6f9fc;
  font-size: 12px;
  cursor: pointer;
  transition:
    background 0.18s,
    box-shadow 0.18s;
}
.ms-role-btn--active {
  background: #e6f3ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.45);
  border-color: rgba(118, 184, 255, 0.9);
}
.ms-close {
  border: 1px solid rgba(170, 190, 215, 0.6);
  background: #fff;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  cursor: pointer;
  font-size: 18px;
  line-height: 30px;
  color: #2a3d52;
}
.ms-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ms-search {
  display: flex;
  gap: 8px;
}
.ms-input {
  flex: 1;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid rgba(170, 190, 215, 0.85);
  font-size: 13px;
}
.ms-search-btn {
  padding: 8px 14px;
  border-radius: 999px;
  border: none;
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}
.ms-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 220px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ms-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-radius: 10px;
  background: #f6f8fc;
  font-size: 13px;
  border: 1px solid transparent;
}
.ms-item--picked {
  background: #e4f7eb;
  border-color: rgba(120, 200, 150, 0.9);
}
.ms-item:hover {
  background: #e6f1ff;
  border-color: rgba(118, 184, 255, 0.9);
}
.ms-item--picked:hover {
  background: #e4f7eb;
  border-color: rgba(120, 200, 150, 0.9);
}
.ms-item__name {
  font-weight: 500;
}
.ms-item__role {
  font-size: 11px;
  color: #5a6d82;
}
.ms-empty {
  text-align: center;
  font-size: 13px;
  color: #6b7c91;
  padding: 14px 0 6px;
}
.ms-picked {
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: #2f4b66;
}
.ms-picked-label {
  font-weight: 600;
}
.ms-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}
.ms-action {
  min-width: 78px;
  padding: 7px 14px;
  border-radius: 999px;
  border: none;
  font-size: 13px;
  cursor: pointer;
}
.ms-action--cancel {
  background: #f3f5f9;
  color: #4a5f7a;
}
.ms-action--confirm {
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #fff;
}
.ms-icon-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 18px;
  width: 24px;
  height: 24px;
  line-height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.ms-icon-btn--add {
  color: #2f7ad1;
}
.ms-icon-btn--remove {
  color: #c04a4a;
}
</style>
