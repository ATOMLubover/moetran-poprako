<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import type { ResMember } from '../api/model/member';
import { useToastStore } from '../stores/toast';
import { getActiveMembers, searchMembersByName } from '../ipc/member';

// 成员筛选条件接口
interface MemberSearchFilters {
  position?: string;
  fuzzyName?: string;
  limit?: number;
}

const emit = defineEmits<{
  (e: 'view-change', view: 'projects' | 'assignments' | 'members'): void;
}>();

const props = defineProps<{
  teamId?: string | null;
  currentView?: 'projects' | 'assignments' | 'members';
  // 来自 PanelView 的筛选条件
  filters?: MemberSearchFilters | undefined;
  // toggle 用于强制重新应用筛选
  shouldApplyFilters?: boolean;
}>();

const toastStore = useToastStore();

// 视图模式：取自父级 currentView
const viewMode = computed(() => props.currentView ?? 'members');

// 无 teamId 时禁用"成员列表"按钮
const canViewMembers = computed(() => !!props.teamId);

function switchView(view: 'projects' | 'assignments' | 'members'): void {
  emit('view-change', view);
}

// 内部成员列表数据
const allMembers = ref<ResMember[]>([]);
const isLoading = ref(false);

// 分页状态
const currentPage = ref(1);
const serverPage = ref(1);
const lastFetchCount = ref(0);

const defaultPageSize = 12;
const pageSize = computed(() => {
  const limit = props.filters?.limit;
  if (typeof limit === 'number' && Number.isFinite(limit) && limit > 0) {
    return limit;
  }

  return defaultPageSize;
});

// 是否还有更多数据
const hasMoreFromServer = computed(() => lastFetchCount.value === pageSize.value);

// 是否有上一页/下一页
const hasPrevPage = computed(() => currentPage.value > 1);
const hasNextPage = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const nextStart = start + pageSize.value;
  return nextStart < allMembers.value.length || hasMoreFromServer.value;
});

// 当前页显示的成员
const displayMembers = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return allMembers.value.slice(start, end);
});

// 总页数
const totalPages = computed(() => {
  return Math.max(1, Math.ceil(allMembers.value.length / pageSize.value));
});

// 上一页
function goPrevPage(): void {
  if (hasPrevPage.value) {
    currentPage.value--;
  }
}

// 下一页
async function goNextPage(): Promise<void> {
  const nextStart = currentPage.value * pageSize.value;
  const remainingLocal = allMembers.value.length - nextStart;

  // 如果本地剩余数据少于 pageSize 且服务端还有更多，则请求更多
  if (remainingLocal < pageSize.value && hasMoreFromServer.value) {
    await fetchMoreFromServer();
  }

  // 检查是否还有数据可以显示
  if (nextStart >= allMembers.value.length) return;

  currentPage.value++;
}

// 从服务端拉取更多数据
async function fetchMoreFromServer(): Promise<void> {
  if (isLoading.value || !props.teamId) return;

  isLoading.value = true;

  try {
    serverPage.value++;

    let list: ResMember[];

    // 判断是否有筛选条件
    const hasFilters = !!(props.filters?.position || props.filters?.fuzzyName);

    if (hasFilters) {
      // 有筛选条件：使用 searchMembersByName
      list = await searchMembersByName({
        teamId: props.teamId,
        position: props.filters?.position as
          | 'translator'
          | 'proofreader'
          | 'typesetter'
          | 'redrawer'
          | 'principal'
          | undefined,
        fuzzyName: props.filters?.fuzzyName,
        page: serverPage.value,
        limit: pageSize.value,
      });
    } else {
      // 无筛选条件：使用 getActiveMembers
      list = await getActiveMembers(props.teamId, serverPage.value, pageSize.value);
    }

    lastFetchCount.value = list.length;

    allMembers.value.push(...list);
  } catch (err) {
    console.error('[MemberList] 获取成员失败:', err);
    toastStore.show(`获取成员失败：${String(err)}`);
  } finally {
    isLoading.value = false;
  }
}

// 初始加载数据
async function fetchAndDisplay(): Promise<void> {
  if (!props.teamId) {
    return;
  }

  isLoading.value = true;

  try {
    serverPage.value = 1;

    let list: ResMember[];

    // 判断是否有筛选条件
    const hasFilters = !!(props.filters?.position || props.filters?.fuzzyName);

    if (hasFilters) {
      // 有筛选条件：使用 searchMembersByName
      list = await searchMembersByName({
        teamId: props.teamId,
        position: props.filters?.position as
          | 'translator'
          | 'proofreader'
          | 'typesetter'
          | 'redrawer'
          | 'principal'
          | undefined,
        fuzzyName: props.filters?.fuzzyName,
        page: 1,
        limit: pageSize.value,
      });
    } else {
      // 无筛选条件：使用 getActiveMembers
      list = await getActiveMembers(props.teamId, 1, pageSize.value);
    }

    lastFetchCount.value = list.length;

    allMembers.value = list;
  } catch (err) {
    console.error('[MemberList] 获取成员失败:', err);
    toastStore.show(`获取成员失败：${String(err)}`);
    allMembers.value = [];
  } finally {
    isLoading.value = false;
  }
}

// 格式化时间
function formatTime(timestamp: number | null): string {
  if (timestamp === null || typeof timestamp === 'undefined') return '从未活跃';

  try {
    // PopRaKo 时间已被后端转换为 unix seconds
    return new Date(timestamp * 1000).toLocaleString('zh-CN');
  } catch (e) {
    return '未知时间';
  }
}

// 初始化加载
onMounted(() => {
  requestAnimationFrame(() => {
    void fetchAndDisplay();
  });
});

// 当 teamId 变化时重新加载
watch(
  () => props.teamId,
  (newVal, oldVal) => {
    if (newVal !== oldVal) {
      allMembers.value = [];
      currentPage.value = 1;
      serverPage.value = 1;
      requestAnimationFrame(() => {
        void fetchAndDisplay();
      });
    }
  }
);

// 当 filters 变化时，重新拉取
watch(
  () => props.filters,
  () => {
    allMembers.value = [];
    currentPage.value = 1;
    serverPage.value = 1;

    requestAnimationFrame(() => {
      void fetchAndDisplay();
    });
  },
  { deep: true }
);

// 当 shouldApplyFilters 切换时，触发刷新
watch(
  () => props.shouldApplyFilters,
  () => {
    allMembers.value = [];
    currentPage.value = 1;
    serverPage.value = 1;
    requestAnimationFrame(() => {
      void fetchAndDisplay();
    });
  }
);
</script>

<template>
  <section class="member-list">
    <header class="member-list__header">
      <div class="member-list__view-toggle view-toggle">
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
          :disabled="!canViewMembers"
        >
          派活列表
        </button>
        <button
          type="button"
          class="view-toggle-btn"
          :class="{ 'view-toggle-btn--active': viewMode === 'members' }"
          @click="switchView('members')"
          :disabled="!canViewMembers"
        >
          成员列表
        </button>
      </div>
      <div class="member-list__controls">
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

    <div class="member-list__content">
      <div v-if="isLoading && allMembers.length === 0" class="member-list__loading">加载中...</div>
      <div v-else-if="!displayMembers.length" class="member-list__empty">暂无成员</div>
      <div v-else class="member-list__grid">
        <div v-for="member in displayMembers" :key="member.memberId" class="member-card">
          <div class="member-card__header">
            <div class="member-card__name">{{ member.username }}</div>
            <div v-if="member.isAdmin" class="member-card__admin-badge">管理员</div>
          </div>
          <div class="member-card__roles">
            <span
              v-if="member.isPrincipal"
              class="member-role-badge member-role--principal"
              title="负责人"
            >
              负责人
            </span>
            <span
              v-if="member.isTranslator"
              class="member-role-badge member-role--translator"
              title="翻译"
            >
              翻译
            </span>
            <span
              v-if="member.isProofreader"
              class="member-role-badge member-role--proofreader"
              title="校对"
            >
              校对
            </span>
            <span
              v-if="member.isTypesetter"
              class="member-role-badge member-role--typesetter"
              title="嵌字"
            >
              嵌字
            </span>
            <span
              v-if="member.isRedrawer"
              class="member-role-badge member-role--redrawer"
              title="美工"
            >
              美工
            </span>
          </div>
          <div class="member-card__active">
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
            {{ formatTime(member.lastActive ?? null) }}
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.member-list {
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

.member-list__header {
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

.member-list__controls {
  display: flex;
  align-items: center;
  gap: 16px;
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

.member-list__content {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
}

.member-list__loading,
.member-list__empty {
  padding: 40px 0;
  text-align: center;
  font-size: 14px;
  color: #4a5f7d;
}

.member-list__grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 14px;
}

.member-card {
  padding: 14px;
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
  gap: 10px;
}

.member-card:hover {
  box-shadow: 0 8px 20px rgba(132, 166, 212, 0.18);
  border-color: rgba(118, 184, 255, 0.6);
  transform: translateY(-2px);
}

.member-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.member-card__name {
  font-size: 16px;
  font-weight: 600;
  color: #294061;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.member-card__admin-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  background: rgb(255, 255, 255);
  color: #6b3a1a;
  border: 1px solid rgba(255, 190, 80, 0.7);
  white-space: nowrap;
}

.member-card__roles {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  min-height: 28px;
}

/* Badge base */
.member-role-badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
}

/* Role-specific styles */
.member-role--principal {
  background: rgba(255, 220, 180, 0.95);
  color: #6b3a1a;
  border: 1px solid rgba(255, 200, 160, 0.7);
}

.member-role--translator {
  background: rgba(237, 235, 137, 0.95);
  color: #836a16;
  border: 1px solid rgba(245, 220, 120, 0.6);
}

.member-role--proofreader {
  background: rgba(255, 235, 240, 0.95);
  color: #7a2536;
  border: 1px solid rgba(255, 180, 200, 0.6);
}

.member-role--typesetter {
  background: rgba(214, 189, 239, 0.95);
  color: #4a2a6a;
  border: 1px solid rgba(205, 180, 255, 0.6);
}

.member-role--redrawer {
  background: rgba(180, 220, 160, 0.95);
  color: #3a5a2a;
  border: 1px solid rgba(160, 200, 140, 0.6);
}

.member-card__active {
  font-size: 11px;
  color: #6b7c8f;
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: auto;
  padding-top: 6px;
  border-top: 1px solid rgba(210, 220, 235, 0.4);
}
</style>
