<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useToastStore } from '../stores/toast';
import { useUserStore } from '../stores/user';
import type { ProjectBasicInfo, PhaseStatus, PhaseChip } from '../api/model/displayProject';
import {
  getUserProjectsEnriched,
  searchUserProjectsEnriched,
  searchTeamProjectsEnriched,
  getTeamProjectsEnriched,
} from '../ipc/project';
import type { ProjectSearchFilters } from '../ipc/project';
import type { ResProjectEnriched } from '../api/model/project';

// 使用共享类型定义（见 src/api/model/displayProject.ts）

// 组件事件：打开详情 / 创建项目
const emit = defineEmits<{
  (e: 'open-detail', projectId: string): void;
  (e: 'create'): void;
}>();

const props = defineProps<{
  // 当前激活的汉化组 id，null/undefined 表示“仅看我自己的项目”
  teamId?: string | null;
  // 来自 PanelView 的筛选条件；空对象或 undefined 表示不启用筛选
  filters?: ProjectSearchFilters | undefined;
}>();

const userStore = useUserStore();

const canCreate = computed(() => {
  // allow create only when a team is selected and the user is admin for that team
  return !!props.teamId && userStore.isAdminFor(props.teamId);
});

// 内部项目列表与加载状态
const innerProjects = ref<(ProjectBasicInfo & { hasPoprako?: boolean })[]>([]);
const isLoading = ref(false);
const listContainerRef = ref<HTMLElement | null>(null);
// resize debounce timer and listener ref
const resizeTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const resizeListener = ref<((this: Window, ev: UIEvent) => any) | null>(null);
// 服务端一次最多拉取多少条，之后前端再根据高度裁剪
const serverLimit = 10;

// 点击详情
function handleOpenDetail(projectId: string): void {
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

// 最终展示数据：始终使用内部拉取的 innerProjects
const displayProjects = computed<(ProjectBasicInfo & { hasPoprako?: boolean })[]>(
  () => innerProjects.value
);

// 将 ResProjectEnriched 转为列表展示 DTO
function mapEnrichedToBasic(
  apiRes: ResProjectEnriched[]
): (ProjectBasicInfo & { hasPoprako?: boolean })[] {
  return apiRes.map(p => {
    const seed = p.translatingStatus ?? p.proofreadingStatus ?? 0;

    const phaseOrder: Array<PhaseChip['phase']> = [
      'translate',
      'proof',
      'typeset',
      'review',
      'publish',
    ];

    const labelMap: Record<PhaseChip['phase'], string> = {
      translate: '翻译',
      proof: '校对',
      typeset: '嵌字',
      review: '监修',
      publish: '发布',
    };

    const phases: PhaseChip[] = phaseOrder.map((phase, i) => {
      const rotate = (seed + i) % 5;
      let status: PhaseStatus = 'unset';
      if (rotate === 1) status = 'pending';
      else if (rotate === 2) status = 'wip';
      else if (rotate === 3) status = 'completed';

      return { phase, status, label: labelMap[phase] } as PhaseChip;
    });

    return {
      // 后端 id 是 UUID，保持为字符串
      id: p.id,
      title: p.name,
      projectSetId: p.projectSet?.id,
      projectSetSerial: p.projsetSerial,
      projectSetIndex: p.projsetIndex,
      hasPoprako: p.hasPoprako,
      phases,
    } satisfies ProjectBasicInfo & { hasPoprako?: boolean };
  });
}

// 根据当前 props.teamId / props.filters 决定调用哪种 IPC
async function fetchAndClamp(): Promise<void> {
  isLoading.value = true;
  try {
    console.log(
      '[ProjectList] fetchAndClamp: requesting',
      serverLimit,
      'items, teamId =',
      props.teamId,
      'filters =',
      props.filters
    );

    let apiRes: ResProjectEnriched[] = [];

    const hasFilters = !!(props.filters && Object.keys(props.filters).length > 0);

    if (props.teamId) {
      // 团队视角：有筛选时走 team search，暂无筛选时使用团队 enriched 列表
      if (hasFilters) {
        apiRes = await searchTeamProjectsEnriched({
          team_id: props.teamId as string,
          ...props.filters,
          page: 1,
          limit: serverLimit,
        });
      } else {
        apiRes = await getTeamProjectsEnriched({
          teamId: props.teamId as string,
          page: 1,
          limit: serverLimit,
        });
      }
    } else {
      // 用户视角
      if (hasFilters) {
        apiRes = await searchUserProjectsEnriched({
          ...props.filters,
          page: 1,
          limit: serverLimit,
        });
      } else {
        apiRes = await getUserProjectsEnriched({ page: 1, limit: serverLimit });
      }
    }
    const all = mapEnrichedToBasic(apiRes);
    innerProjects.value = all;

    // 下一帧再测量，确保 DOM 已更新；如果此时 DOM 未挂载，跳过裁剪但保留数据
    requestAnimationFrame(() => {
      const container = listContainerRef.value;
      if (!container) {
        console.log('[ProjectList] fetchAndClamp: container not mounted yet, skip clamp');
        return;
      }

      const scroll = container.closest('.projects-scroll') as HTMLElement | null;
      const host = scroll ?? container;
      const hostRect = host.getBoundingClientRect();
      const items = host.querySelectorAll('.project-list__item');
      if (!items.length) {
        console.log('[ProjectList] fetchAndClamp: no items rendered');
        return;
      }

      const firstItemRect = (items[0] as HTMLElement).getBoundingClientRect();
      const itemHeight = firstItemRect.height;
      const verticalGap = 16; // 来自 .project-list__items 的 gap

      const totalItemBlock = itemHeight + verticalGap;
      const safePadding = 8;
      const usableHeight = hostRect.height - safePadding;
      const maxItems = Math.max(1, Math.floor(usableHeight / totalItemBlock));

      console.log(
        '[ProjectList] clamp: hostHeight=',
        hostRect.height,
        'itemHeight=',
        itemHeight,
        'gap=',
        verticalGap,
        'maxItems=',
        maxItems
      );

      if (innerProjects.value.length > maxItems) {
        innerProjects.value = innerProjects.value.slice(0, maxItems);
      }
    });
  } catch (err) {
    console.error('[ProjectList] 获取用户项目失败:', err);
    innerProjects.value = [];
    try {
      const toastStore = useToastStore();
      // 给用户友好的提示（网络或后端服务不可用）
      toastStore.show(`获取项目失败：${String(err)}`);
    } catch (e) {
      // 如果 toast 无法使用，继续静默处理
      console.debug('ProjectList toast failed', e);
    }
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  requestAnimationFrame(() => {
    void fetchAndClamp();
  });
  // 当窗口尺寸变化时，debounce 后重新裁剪以适配空间
  const onResize = () => {
    if (resizeTimer.value) clearTimeout(resizeTimer.value);
    resizeTimer.value = setTimeout(() => {
      requestAnimationFrame(() => {
        void fetchAndClamp();
      });
    }, 150);
  };

  window.addEventListener('resize', onResize);
  // store listener so we can remove it on unmount
  resizeListener.value = onResize;
});

// 当 filters 变化时，重新拉取并裁剪
watch(
  () => props.filters,
  () => {
    requestAnimationFrame(() => {
      void fetchAndClamp();
    });
  },
  { deep: true }
);

// 当 teamId 变化时也需重新拉取
watch(
  () => props.teamId,
  (newVal, oldVal) => {
    console.log('[ProjectList] teamId changed:', oldVal, '->', newVal);
    requestAnimationFrame(() => {
      void fetchAndClamp();
    });
  }
);

onBeforeUnmount(() => {
  // 清理 resize 监听器
  if (resizeListener.value) {
    window.removeEventListener('resize', resizeListener.value);
    resizeListener.value = null;
  }
  if (resizeTimer.value) {
    clearTimeout(resizeTimer.value);
    resizeTimer.value = null;
  }
});
</script>

<template>
  <section class="project-list">
    <header class="project-list__header">
      <h3 class="project-list__title">当前项目</h3>
      <div
        class="project-list__header-actions"
        :class="{ 'project-list__header-actions--locked': !canCreate }"
      >
        <button
          type="button"
          class="project-list__publish"
          @click="
            () => {
              /* TODO: 发布项目回调 */
            }
          "
          :disabled="isLoading || !canCreate"
        >
          发布项目
        </button>
        <button
          type="button"
          class="project-list__create"
          @click="handleCreateProject"
          :disabled="isLoading || !canCreate"
        >
          创建新项目
        </button>
        <!-- <span v-if="!canCreate" class="project-list__locked-note">🔒 仅团队管理员可创建</span> -->
      </div>
    </header>

    <div class="project-list__content" ref="listContainerRef">
      <ul class="project-list__items" v-if="displayProjects.length > 0">
        <li
          v-for="item in displayProjects"
          :key="String(item.id)"
          class="project-list__item"
          :class="{ 'project-list__item--poprako': item.hasPoprako }"
        >
          <div class="project-list__item-main">
            <h3 class="project-list__item-title">
              <span v-if="item.hasPoprako" class="project-list__tag-poprako">PopRaKo</span>
              {{ item.title }}
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
      <div v-else-if="isLoading" class="project-list__loading">加载中...</div>
      <div v-else class="project-list__empty">暂无项目</div>
    </div>
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

.project-list__header-actions {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}

/* When the current user cannot create projects for the selected team,
   show the actions area with a muted/locked appearance while keeping layout. */
.project-list__header-actions--locked {
  opacity: 0.95;
}

.project-list__header-actions--locked .project-list__create,
.project-list__header-actions--locked .project-list__publish {
  background: linear-gradient(135deg, #f6f8fa, #fbfdff);
  color: #7a8796;
  border-color: rgba(200, 208, 218, 0.6);
  box-shadow: none;
}

.project-list__locked-note {
  margin-left: 6px;
  font-size: 13px;
  color: #6d7a8a;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.project-list__title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: #1f2e43;
}

.project-list__create,
.project-list__publish {
  border-radius: 10px;
  padding: 8px 12px;
  font-size: 14px;
  font-weight: 600;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  cursor: pointer;
  box-shadow: 0 6px 16px rgba(118, 184, 255, 0.06);
  transition:
    transform 0.14s ease,
    box-shadow 0.14s ease,
    background 0.12s ease;
}

.project-list__create:hover:not(:disabled),
.project-list__publish:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.12);
  background: #eef6ff;
}

.project-list__create:disabled,
.project-list__publish:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
}

.project-list__create:not(:disabled):hover,
.project-list__publish:not(:disabled):hover {
  transform: translateY(-2px);
  box-shadow: 0 14px 30px rgba(118, 184, 255, 0.42);
}

.project-list__publish {
  padding-inline: 14px;
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

.project-list__item--poprako {
  border-color: rgba(140, 205, 170, 0.9);
  box-shadow: 0 12px 26px rgba(120, 190, 160, 0.26);
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

.project-list__tag-poprako {
  display: inline-block;
  margin-right: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  background: rgba(210, 244, 225, 0.96);
  color: #1e6042;
  border: 1px solid rgba(140, 205, 170, 0.9);
  vertical-align: middle;
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
  border: 1px solid rgba(124, 205, 182, 0.25);
  border-radius: 12px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  background: linear-gradient(135deg, rgba(124, 205, 182, 0.14), rgba(146, 214, 222, 0.06));
  color: #114f45;
  box-shadow: none;
  transition:
    transform 0.18s ease,
    box-shadow 0.12s ease,
    background 0.12s ease;
}

.project-list__detail-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 18px rgba(124, 205, 182, 0.08);
  background: #eafaf6;
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
