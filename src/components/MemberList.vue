<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import type { ResMember } from '../api/model/member';
import { useToastStore } from '../stores/toast';
import { getActiveMembers } from '../ipc/member';

const emit = defineEmits<{
  (e: 'view-change', view: 'projects' | 'assignments' | 'members'): void;
}>();

const props = defineProps<{
  teamId?: string | null;
  currentView?: 'projects' | 'assignments' | 'members';
}>();

const toastStore = useToastStore();
// 视图模式：取自父级 currentView
const viewMode = computed(() => props.currentView ?? 'members');

// 无 teamId 时禁用"成员列表"按钮
const canViewMembers = computed(() => !!props.teamId);

function switchView(view: 'projects' | 'assignments' | 'members'): void {
  emit('view-change', view);
}

// 当前页成员数据（按页从服务器拉取）
const currentPageMembers = ref<ResMember[]>([]);
const isLoading = ref(false);

// 每页显示项目数（后端 limit）
const pageSize = 12;

// 分页状态
const currentPage = ref(1);

// 乐观地判断是否可能有下一页：当后端返回的数量等于 limit 时，认为可能还有下一页
const maybeHasNext = ref<boolean>(false);

// 是否有上一页
const hasPrevPage = computed(() => currentPage.value > 1);

// 请求特定页的数据并返回结果（不修改 currentPage）
async function fetchMembersPage(page: number): Promise<ResMember[]> {
  if (!props.teamId) return [];

  isLoading.value = true;
  try {
    const data = await getActiveMembers(props.teamId, page, pageSize);
    return data;
  } catch (err) {
    console.error('[MemberList] 获取成员列表失败:', err);
    toastStore.show(`获取成员列表失败：${String(err)}`);
    return [];
  } finally {
    isLoading.value = false;
  }
}

// 将某页数据设置为当前页显示
function applyPageData(page: number, data: ResMember[]): void {
  currentPage.value = page;
  currentPageMembers.value = data;
  // 如果返回数量等于 limit，则可能还有下一页
  maybeHasNext.value = data.length === pageSize;
}

// 上一页：尝试请求上一页并应用
async function goPrevPage(): Promise<void> {
  if (!hasPrevPage.value || isLoading.value) return;

  const target = currentPage.value - 1;
  const data = await fetchMembersPage(target);
  // 即便上一页返回 0（异常情况），也切回上一页并显示为空
  applyPageData(target, data);
}

// 下一页：如果后端明确没有更多（maybeHasNext=false）则直接提示；否则尝试请求下一页
async function goNextPage(): Promise<void> {
  if (isLoading.value) return;

  if (!maybeHasNext.value && currentPage.value !== 0) {
    // 已知没有下一页
    toastStore.show('没有下一页');
    return;
  }

  const target = currentPage.value + 1;
  const data = await fetchMembersPage(target);

  if (!data.length) {
    // 后端返回 0，说明实际上没有下一页
    maybeHasNext.value = false;
    toastStore.show('没有下一页了');
    return;
  }

  // 有数据，切到下一页
  applyPageData(target, data);
}

// 初始化：加载第一页并设置状态
onMounted(() => {
  void (async () => {
    if (!props.teamId) return;

    const data = await fetchMembersPage(1);
    applyPageData(1, data);
  })();
});

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
// 当 teamId 变化时重置并加载第一页
watch(
  () => props.teamId,
  () => {
    void (async () => {
      currentPage.value = 1;
      if (!props.teamId) {
        currentPageMembers.value = [];
        maybeHasNext.value = false;
        return;
      }

      const data = await fetchMembersPage(1);
      applyPageData(1, data);
    })();
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
          <span class="page-indicator">{{ currentPage }}</span>
          <button
            type="button"
            class="pagination-btn"
            @click="goNextPage"
            :disabled="isLoading || !maybeHasNext"
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
      <div v-if="isLoading" class="member-list__loading">加载中...</div>
      <div v-else-if="!currentPageMembers.length" class="member-list__empty">暂无成员</div>
      <div v-else class="member-list__grid">
        <div v-for="member in currentPageMembers" :key="member.memberId" class="member-card">
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
