<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type {
  ShownProjectInfo,
  PhaseChip,
  PhaseStatus,
  WorkPhase,
} from '../api/model/shownProject';
import type { TeamShownProject } from '../ipc/project';
import { listTeamShownProjects } from '../ipc/project';
import type { ResMember } from '../api/model/member';
import { useToastStore } from '../stores/toast';

const props = defineProps<{
  teamId: string;
}>();

const emit = defineEmits<{
  (e: 'exit'): void;
  (
    e: 'open-detail',
    payload: {
      id: string;
      title: string;
      projsetName: string | null;
      projsetIndex: number | null;
      principals?: string[];
      totalMarkers: number | null;
      totalTranslated: number | null;
      totalChecked: number | null;
      translatingStatus: number | null;
      proofreadingStatus: number | null;
      typesettingStatus: number | null;
      reviewingStatus: number | null;
      translators: string[];
      proofreaders: string[];
      letterers: string[];
      reviewers: string[];
      members?: ResMember[];
      isPublished?: boolean;
      hasPoprako?: boolean;
      teamId?: string;
    }
  ): void;
}>();

const serverLimit = 20;
const toastStore = useToastStore();

const allEntries = ref<ShownProjectInfo[]>([]);
const innerEntries = ref<ShownProjectInfo[]>([]);
const currentStartIndex = ref(0);
const isLoading = ref(false);
const errorMessage = ref('');
const listContainerRef = ref<HTMLElement | null>(null);
const resizeTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const resizeListener = ref<((this: Window, ev: UIEvent) => void) | null>(null);
const serverPage = ref(1);
const lastFetchCount = ref(0);
const reachedEnd = ref(false);
const fixedPageSize = ref(0);

const hasTeam = computed(() => !!props.teamId);
const displayEntries = computed<ShownProjectInfo[]>(() => innerEntries.value);
const currentPageNumber = computed(() => {
  if (fixedPageSize.value === 0) {
    return 1;
  }

  return Math.floor(currentStartIndex.value / fixedPageSize.value) + 1;
});
const pageLabel = computed(() => `第 ${currentPageNumber.value} 页`);
const canPrev = computed(() => currentStartIndex.value > 0);
const canNext = computed(() => {
  const pageSize = fixedPageSize.value || 1;
  const nextStartIndex = currentStartIndex.value + pageSize;

  const localHasNext = nextStartIndex < allEntries.value.length;

  return localHasNext || !reachedEnd.value;
});

watch(
  () => props.teamId,
  newVal => {
    resetAndFetch(newVal);
  }
);

function resetAndFetch(teamId?: string): void {
  allEntries.value = [];
  innerEntries.value = [];
  currentStartIndex.value = 0;
  serverPage.value = 1;
  lastFetchCount.value = 0;
  reachedEnd.value = false;
  fixedPageSize.value = 0;

  if (!teamId) {
    errorMessage.value = '';
    return;
  }

  requestAnimationFrame(() => {
    void fetchAndClamp();
  });
}

async function fetchAndClamp(): Promise<void> {
  if (!props.teamId) {
    allEntries.value = [];
    innerEntries.value = [];
    return;
  }

  isLoading.value = true;
  errorMessage.value = '';

  try {
    console.log('[ShownOverview] FetchAndClamp start', {
      teamId: props.teamId,
      page: 1,
      limit: serverLimit,
    });

    const rawList = await listTeamShownProjects({
      teamId: props.teamId,
      page: 1,
      limit: serverLimit,
    });

    lastFetchCount.value = rawList.length;
    serverPage.value = 1;
    allEntries.value = rawList.map(item => mapToShownInfo(item));
    currentStartIndex.value = 0;
    reachedEnd.value = false;

    console.log('[ShownOverview] FetchAndClamp success', {
      received: rawList.length,
      totalCached: allEntries.value.length,
    });

    clampAndDisplay('fetchAndClamp');
  } catch (err) {
    console.error('Failed to load shown projects:', err);
    allEntries.value = [];
    innerEntries.value = [];
    lastFetchCount.value = 0;
    errorMessage.value = '加载纵览表格失败，请稍后重试';
  } finally {
    isLoading.value = false;
  }
}

function mapToShownInfo(item: TeamShownProject): ShownProjectInfo {
  const translatorNames = pickMembers(item.members, m => m.isTranslator);
  const proofreaderNames = pickMembers(item.members, m => m.isProofreader);
  const typesetterNames = pickMembers(item.members, m => m.isTypesetter);
  const redrawerNames = pickMembers(item.members, m => m.isRedrawer);
  const reviewerNames = pickMembers(item.members, m => m.isPrincipal);

  const phases: PhaseChip[] = [
    buildPhaseChip('translate', '翻译', item.translatingStatus, translatorNames),
    buildPhaseChip('proof', '校对', item.proofreadingStatus, proofreaderNames),
    buildPhaseChip('typeset', '嵌字', item.typesettingStatus, typesetterNames),
    buildPhaseChip('redraw', '美工', null, redrawerNames),
    buildPhaseChip('review', '监修', item.reviewingStatus, reviewerNames),
  ];

  return {
    id: item.id,
    title: item.name,
    description: item.description ?? null,
    phases,
    translatedSourceCount: item.translatedSourceCount ?? 0,
    proofreadSourceCount: item.proofreadSourceCount ?? 0,
    members: item.members,
    projsetSerial: item.projsetSerial ?? null,
    projsetIndex: item.projsetIndex ?? null,
    isPublished: item.isPublished,
    translatingStatus: item.translatingStatus,
    proofreadingStatus: item.proofreadingStatus,
    typesettingStatus: item.typesettingStatus,
    reviewingStatus: item.reviewingStatus,
    translatorNames,
    proofreaderNames,
    typesetterNames,
    redrawerNames,
    reviewerNames,
  };
}

function pickMembers(source: ResMember[], predicate: (m: ResMember) => boolean): string[] {
  return source.filter(predicate).map(m => m.username);
}

function buildPhaseChip(
  phase: WorkPhase,
  label: string,
  statusCode: number | null,
  memberNames: string[]
): PhaseChip {
  return {
    phase,
    label,
    status: resolvePhaseStatus(statusCode),
    memberNames,
  };
}

function resolvePhaseStatus(code: number | null | undefined): PhaseStatus {
  if (code === null || typeof code === 'undefined') {
    return 'unset';
  }

  if (code <= 0) {
    return 'pending';
  }

  if (code === 1) {
    return 'wip';
  }

  if (code >= 2) {
    return 'completed';
  }

  return 'unset';
}

function clampAndDisplay(reason: string): void {
  void nextTick().then(() => {
    requestAnimationFrame(() => {
      const container = listContainerRef.value;

      if (!container) {
        console.log('[ShownOverview] Clamp skipped: container missing', { reason });
        return;
      }

      if (allEntries.value.length === 0) {
        innerEntries.value = [];
        console.log('[ShownOverview] Clamp: no entries cached', { reason });
        return;
      }

      const availableItems = allEntries.value.slice(currentStartIndex.value);
      innerEntries.value = availableItems;

      void nextTick().then(() => {
        requestAnimationFrame(() => {
          const host = container.closest('.shown-overview') ?? container;
          const items = host.querySelectorAll('.shown-entry');

          if (!items.length) {
            console.log('[ShownOverview] Clamp: no rendered items', { reason });
            return;
          }

          let visibleCount = 0;

          for (let i = 0; i < items.length; i++) {
            const itemRect = (items[i] as HTMLElement).getBoundingClientRect();
            const bottomWithPadding = itemRect.bottom + 20;

            if (bottomWithPadding > window.innerHeight) {
              break;
            }

            visibleCount += 1;
          }

          const finalCount = Math.max(1, Math.min(visibleCount, serverLimit));

          const needFetchToFill =
            finalCount > availableItems.length && !reachedEnd.value && !isLoading.value;

          if (needFetchToFill) {
            console.log('[ShownOverview] Clamp: need fetch to fill view', {
              reason,
              finalCount,
              available: availableItems.length,
            });

            void fetchMoreFromServer('clamp-fill').then(fetched => {
              if (fetched) {
                clampAndDisplay('after-fetch-fill');
              }
            });

            return;
          }

          if (innerEntries.value.length > finalCount) {
            innerEntries.value = availableItems.slice(0, finalCount);
          }

          if (fixedPageSize.value === 0 && innerEntries.value.length > 0) {
            fixedPageSize.value = innerEntries.value.length;
            console.log('[ShownOverview] Fixed page size determined', {
              pageSize: fixedPageSize.value,
            });
          }

          console.log('[ShownOverview] Clamp done', {
            reason,
            visibleCount,
            finalCount,
            cached: allEntries.value.length,
            showing: innerEntries.value.length,
            start: currentStartIndex.value,
            fixedPageSize: fixedPageSize.value,
          });
        });
      });
    });
  });
}

function goPrev(): void {
  if (!canPrev.value) {
    return;
  }

  const pageSize = fixedPageSize.value || serverLimit;
  currentStartIndex.value = Math.max(0, currentStartIndex.value - pageSize);

  console.log('[ShownOverview] GoPrev', {
    pageSize,
    currentStartIndex: currentStartIndex.value,
  });

  clampAndDisplay('goPrev');
}

async function fetchMoreFromServer(trigger: string): Promise<boolean> {
  if (isLoading.value || !props.teamId) {
    console.log('[ShownOverview] FetchMore skipped', {
      trigger,
      isLoading: isLoading.value,
      teamId: props.teamId,
    });
    return false;
  }

  const nextPage = serverPage.value + 1;
  isLoading.value = true;

  try {
    console.log('[ShownOverview] FetchMore start', {
      trigger,
      nextPage,
      limit: serverLimit,
    });

    const rawList = await listTeamShownProjects({
      teamId: props.teamId,
      page: nextPage,
      limit: serverLimit,
    });

    if (rawList.length === 0) {
      lastFetchCount.value = 0;
      toastStore.show('已到最后一页');
      reachedEnd.value = true;
      console.log('[ShownOverview] FetchMore reached end', { trigger, nextPage });
      return false;
    }

    serverPage.value = nextPage;
    lastFetchCount.value = rawList.length;
    const mapped = rawList.map(item => mapToShownInfo(item));

    allEntries.value = [...allEntries.value, ...mapped];

    console.log('[ShownOverview] FetchMore success', {
      trigger,
      received: rawList.length,
      totalCached: allEntries.value.length,
    });

    return true;
  } catch (err) {
    console.error('Failed to fetch more shown projects:', err);
    toastStore.show('获取下一页失败，请稍后再试');

    return false;
  } finally {
    isLoading.value = false;
  }
}

async function goNext(): Promise<void> {
  if (!canNext.value) {
    return;
  }

  const pageSize = fixedPageSize.value || 1;
  const nextStartIndex = currentStartIndex.value + pageSize;
  const remainingLocal = allEntries.value.length - nextStartIndex;
  const localHasNext = nextStartIndex < allEntries.value.length;

  console.log('[ShownOverview] GoNext attempt', {
    pageSize,
    nextStartIndex,
    remainingLocal,
    cached: allEntries.value.length,
    reachedEnd: reachedEnd.value,
    fixedPageSize: fixedPageSize.value,
  });

  if (!localHasNext && reachedEnd.value) {
    toastStore.show('已到最后一页');
    return;
  }

  if (!localHasNext) {
    const fetched = await fetchMoreFromServer('goNext');

    if (!fetched && nextStartIndex >= allEntries.value.length) {
      return;
    }
  }

  if (nextStartIndex >= allEntries.value.length) {
    toastStore.show('已到最后一页');
    return;
  }

  currentStartIndex.value = nextStartIndex;

  console.log('[ShownOverview] GoNext success', {
    currentStartIndex: currentStartIndex.value,
  });

  clampAndDisplay('goNext');
}

function handleExit(): void {
  emit('exit');
}

function openDetail(info: ShownProjectInfo): void {
  emit('open-detail', {
    id: info.id,
    title: info.title,
    projsetName: null,
    projsetIndex: info.projsetIndex ?? null,
    principals: info.reviewerNames,
    totalMarkers: info.translatedSourceCount,
    totalTranslated: info.translatedSourceCount,
    totalChecked: info.proofreadSourceCount,
    translatingStatus: info.translatingStatus,
    proofreadingStatus: info.proofreadingStatus,
    typesettingStatus: info.typesettingStatus,
    reviewingStatus: info.reviewingStatus,
    translators: info.translatorNames,
    proofreaders: info.proofreaderNames,
    letterers: info.typesetterNames,
    reviewers: info.reviewerNames,
    members: info.members,
    isPublished: info.isPublished,
    hasPoprako: true,
    teamId: props.teamId,
  });
}

onMounted(() => {
  if (hasTeam.value) {
    requestAnimationFrame(() => {
      void fetchAndClamp();
    });
  }

  const onResize = () => {
    if (resizeTimer.value) {
      clearTimeout(resizeTimer.value);
    }

    resizeTimer.value = setTimeout(() => {
      requestAnimationFrame(() => {
        console.log('[ShownOverview] Resize trigger clamp');
        clampAndDisplay('resize');
      });
    }, 150);
  };

  window.addEventListener('resize', onResize);
  resizeListener.value = onResize;
});

onBeforeUnmount(() => {
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
  <section class="shown-overview">
    <header class="overview-head">
      <div class="head-text">
        <h2 class="head-title">PopRaKo 纵览表格</h2>
      </div>
      <div class="overview-head__actions">
        <span class="page-indicator">{{ pageLabel }}</span>
        <div class="page-buttons">
          <button class="nav-btn" :disabled="!canPrev" @click="goPrev">上一页</button>
          <button class="nav-btn" :disabled="!canNext" @click="goNext">下一页</button>
        </div>
        <button class="overview-back" @click="handleExit">返回</button>
      </div>
    </header>

    <div class="overview-body" ref="listContainerRef">
      <div v-if="!hasTeam" class="overview-placeholder">请选择汉化组以查看纵览表格</div>
      <div v-else-if="isLoading" class="overview-placeholder">载入纵览数据...</div>
      <div v-else-if="errorMessage" class="overview-placeholder">{{ errorMessage }}</div>
      <div v-else-if="displayEntries.length === 0" class="overview-placeholder">暂无项目</div>
      <div v-else class="overview-grid">
        <article v-for="item in displayEntries" :key="item.id" class="shown-entry">
          <div class="col col-title">
            <div class="title-row">
              <div class="entry-title">{{ item.title }}</div>
              <span class="entry-publish" :class="{ 'entry-publish--done': item.isPublished }">{{
                item.isPublished ? '发' : '发'
              }}</span>
            </div>
          </div>

          <div class="col col-translate">
            <div
              :class="[
                'phase-chip',
                `phase-chip--${item.phases.find((p: PhaseChip) => p.phase === 'translate')?.status ?? 'unset'}`,
              ]"
            >
              <div class="phase-chip__members">
                {{
                  item.phases.find((p: PhaseChip) => p.phase === 'translate')?.memberNames.length
                    ? item.phases
                        .find((p: PhaseChip) => p.phase === 'translate')
                        ?.memberNames.join('、')
                    : '-'
                }}
              </div>
            </div>
          </div>

          <div class="col col-proof">
            <div
              :class="[
                'phase-chip',
                `phase-chip--${item.phases.find((p: PhaseChip) => p.phase === 'proof')?.status ?? 'unset'}`,
              ]"
            >
              <div class="phase-chip__members">
                {{
                  item.phases.find((p: PhaseChip) => p.phase === 'proof')?.memberNames.length
                    ? item.phases
                        .find((p: PhaseChip) => p.phase === 'proof')
                        ?.memberNames.join('、')
                    : '-'
                }}
              </div>
            </div>
          </div>

          <div class="col col-typeset">
            <div
              :class="[
                'phase-chip',
                `phase-chip--${item.phases.find((p: PhaseChip) => p.phase === 'typeset')?.status ?? 'unset'}`,
              ]"
            >
              <div class="phase-chip__members">
                {{
                  item.phases.find((p: PhaseChip) => p.phase === 'typeset')?.memberNames.length
                    ? item.phases
                        .find((p: PhaseChip) => p.phase === 'typeset')
                        ?.memberNames.join('、')
                    : '-'
                }}
              </div>
            </div>
          </div>

          <div class="col col-redraw">
            <div
              :class="[
                'phase-chip',
                `phase-chip--${item.phases.find((p: PhaseChip) => p.phase === 'redraw')?.status ?? 'unset'}`,
              ]"
            >
              <div class="phase-chip__members">
                {{
                  item.phases.find((p: PhaseChip) => p.phase === 'redraw')?.memberNames.length
                    ? item.phases
                        .find((p: PhaseChip) => p.phase === 'redraw')
                        ?.memberNames.join('、')
                    : '-'
                }}
              </div>
            </div>
          </div>

          <div class="col col-review">
            <div
              :class="[
                'phase-chip',
                `phase-chip--${item.phases.find((p: PhaseChip) => p.phase === 'review')?.status ?? 'unset'}`,
              ]"
            >
              <div class="phase-chip__members">
                {{
                  item.phases.find((p: PhaseChip) => p.phase === 'review')?.memberNames.length
                    ? item.phases
                        .find((p: PhaseChip) => p.phase === 'review')
                        ?.memberNames.join('、')
                    : '-'
                }}
              </div>
            </div>
          </div>

          <div class="col col-counts">
            <div class="counts-and-status">
              <div class="counts-left">
                <div class="count-row">
                  <span class="count-chip count-chip--trans"
                    >已翻 {{ item.translatedSourceCount }}</span
                  >
                </div>
                <div class="count-row">
                  <span class="count-chip count-chip--proof"
                    >已校 {{ item.proofreadSourceCount }}</span
                  >
                </div>
              </div>
            </div>
          </div>

          <div class="col col-actions">
            <div class="status-row">
              <span
                :class="[
                  'status-chip',
                  `status-chip--${item.phases.find((p: PhaseChip) => p.phase === 'translate')?.status ?? 'unset'}`,
                ]"
                >翻</span
              >
              <span
                :class="[
                  'status-chip',
                  `status-chip--${item.phases.find((p: PhaseChip) => p.phase === 'proof')?.status ?? 'unset'}`,
                ]"
                >校</span
              >
              <span
                :class="[
                  'status-chip',
                  `status-chip--${item.phases.find((p: PhaseChip) => p.phase === 'typeset')?.status ?? 'unset'}`,
                ]"
                >嵌</span
              >
              <span
                :class="[
                  'status-chip',
                  `status-chip--${item.phases.find((p: PhaseChip) => p.phase === 'review')?.status ?? 'unset'}`,
                ]"
                >监</span
              >
            </div>

            <div class="actions-row">
              <button class="entry-detail-btn" @click="openDetail(item)">详情</button>
            </div>
          </div>
        </article>
      </div>
    </div>
  </section>
</template>

<style scoped>
.shown-overview {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(150, 180, 210, 0.22);
  border-radius: 12px;
  padding: 12px 14px; /* reduce outer padding */
  box-shadow: 0 10px 24px rgba(110, 160, 210, 0.18);
  min-height: 0;
  box-sizing: border-box;
}

.overview-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  flex-wrap: wrap;
}

.head-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.head-title {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: #1f3452;
}

.head-subtitle {
  margin: 0;
  font-size: 13px;
  color: #58708f;
}

.overview-head__actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.page-indicator {
  font-size: 14px;
  font-weight: 600;
  color: #28405c;
}

.page-buttons {
  display: inline-flex;
  gap: 8px;
}

.nav-btn {
  border: 1px solid rgba(118, 184, 255, 0.4);
  background: #f4f9ff;
  color: #2f5a8f;
  padding: 6px 14px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    background 0.12s ease;
}

.nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}

.nav-btn:not(:disabled):hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.22);
}

.overview-back {
  border: 1px solid rgba(124, 205, 182, 0.5);
  background: linear-gradient(135deg, rgba(124, 205, 182, 0.1), rgba(146, 214, 222, 0.18));
  color: #0e514a;
  padding: 6px 18px;
  border-radius: 12px;
  font-weight: 600;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease;
}

.overview-back:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 28px rgba(124, 205, 182, 0.35);
}

.overview-body {
  margin-top: 18px;
  flex: 1;
  min-height: 0;
}

.overview-placeholder {
  height: 100%;
  border-radius: 16px;
  border: 1px dashed rgba(156, 184, 214, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  color: #4d6584;
  background: rgba(248, 252, 255, 0.9);
}

.overview-grid {
  display: block;
  gap: 6px;
}

.shown-entry {
  display: grid;
  grid-template-columns: 2fr repeat(5, 1fr) 120px 140px;
  align-items: center;
  gap: 8px;
  border: 1px solid rgba(200, 208, 218, 0.9);
  border-radius: 8px;
  padding: 6px 8px;
  background: transparent;
  box-sizing: border-box;
}

.col {
  display: flex;
  align-items: center;
}

.col .phase-chip {
  width: 100%;
}

.col-title {
  font-weight: 600;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.entry-header {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.entry-title {
  margin: 0;
  font-size: 14px; /* smaller title */
  font-weight: 600;
  color: #1f2f46;
}

.entry-serial {
  font-size: 12px;
  color: #6a819d;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(98, 130, 166, 0.12);
}

.col-translate,
.col-proof,
.col-typeset,
.col-redraw,
.col-review {
  padding: 0 6px;
}

.col-counts {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  padding-right: 8px;
}

.count-row {
  line-height: 1;
}

.phase-chip {
  border-radius: 6px;
  padding: 2px 6px; /* compact member area */
  background: #ffffff; /* pure white background for member chips */
  border: 2px solid rgba(200, 208, 218, 0.9); /* neutral gray border, thicker */
  display: flex;
  flex-direction: row; /* inline: label and members on same line */
  align-items: center;
  gap: 6px;
  font-size: 10px;
}

.phase-chip__label {
  font-size: 10px; /* compact label */
}

/* append Chinese colon after the label so it reads "翻译：name" */
.phase-chip__label::after {
  /* Chinese colon with non-breaking space to keep the first name attached */
  content: '：\00A0';
  margin-left: 0;
}

.phase-chip__members {
  font-size: 12px;
  color: #294b6b;
  white-space: normal; /* allow natural wrapping */
  overflow-wrap: break-word; /* avoid breaking inside usernames arbitrarily */
  word-break: normal;
  text-align: center; /* center member names */
  flex: 1 1 auto; /* take available horizontal space */
}

.phase-chip__empty {
  font-size: 12px;
  color: #9aa9bc;
}

.phase-chip--pending,
.phase-chip--wip,
.phase-chip--completed,
.phase-chip--unset {
  /* keep member chips visually neutral: white bg and gray border regardless of status */
  background: #ffffff;
  border-color: rgba(200, 208, 218, 0.9);
}

.count-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.count-chip {
  padding: 4px 12px;
  padding: 1px 6px;
  border-radius: 999px;
  font-size: 9px;
  font-weight: 600;
  color: #1e405d;
  background: rgba(226, 239, 255, 0.8);
}
.count-chip--trans {
  background: rgba(255, 249, 220, 0.95); /* light yellow */
  color: #6a5400;
  border: 1px solid rgba(230, 215, 150, 0.6);
}
.count-chip--proof {
  background: rgba(255, 238, 242, 0.95); /* light pink */
  color: #8b3b4b;
  border: 1px solid rgba(240, 200, 210, 0.6);
}

/* status chips (small, next to counts) */
.counts-and-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.counts-left {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.status-row {
  display: flex;
  gap: 6px;
  align-items: center;
  justify-content: flex-end;
  width: 100%;
}

.status-chip {
  padding: 0 6px;
  border-radius: 999px;
  font-size: 9px;
  font-weight: 700;
  color: #1b3550;
}

/* make each status chip flex so total chips fill the column width (match Details button) */
.status-row .status-chip {
  flex: 1 1 0;
  display: inline-flex;
  justify-content: center;
  padding: 2px 4px;
  box-sizing: border-box;
}

.status-chip--pending {
  background: rgba(255, 238, 210, 0.95);
  color: #7a4a00;
  border: 1px solid rgba(255, 199, 141, 0.8);
}

.status-chip--wip {
  background: rgba(210, 236, 255, 0.95);
  color: #174a6f;
  border: 1px solid rgba(118, 184, 255, 0.8);
}

.status-chip--completed {
  background: rgba(210, 244, 225, 0.95);
  color: #1f6740;
  border: 1px solid rgba(120, 200, 160, 0.9);
}

.status-chip--unset {
  background: rgba(240, 242, 245, 0.95);
  color: #5b6d83;
  border: 1px solid rgba(200, 208, 218, 0.7);
}

.col-actions {
  display: flex;
  flex-direction: column;
  align-items: stretch; /* ensure child rows fill the column width */
  gap: 6px;
}

.actions-row {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
}

.entry-detail-btn {
  border: 1px solid rgba(124, 205, 182, 0.7);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: linear-gradient(135deg, rgba(124, 205, 182, 0.12), rgba(146, 214, 222, 0.2));
  color: #0f5245;
  transition:
    transform 0.12s ease,
    box-shadow 0.12s ease;
  width: 100%;
  box-sizing: border-box;
}

.entry-detail-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 26px rgba(124, 205, 182, 0.3);
}

.entry-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.entry-publish {
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid rgba(200, 208, 218, 0.8);
  color: #5b6d83;
}

.entry-publish--done {
  border-color: rgba(120, 200, 160, 0.9);
  color: #1f5d35;
}

@media (max-width: 1280px) {
  .shown-overview {
    padding: 16px;
  }

  .shown-entry {
    grid-template-columns: 1fr;
  }

  .shown-entry__right {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
}

/* Override: make status chips and detail areas even smaller and denser */
.shown-overview .phase-chip {
  padding: 1px 4px !important;
  font-size: 9px !important;
  gap: 1px !important;
  border-radius: 6px !important;
}

.shown-overview .phase-chip__label,
.shown-overview .phase-chip__members,
.shown-overview .phase-chip__empty {
  font-size: 9px !important;
}

.shown-overview .count-chip {
  padding: 0 6px !important;
  font-size: 8px !important;
}

.shown-overview .entry-detail-btn {
  padding: 6px 8px !important;
  font-size: 11px !important;
  border-radius: 8px !important;
}

.shown-overview .shown-entry {
  padding: 4px !important;
  gap: 4px !important;
}

.shown-overview .entry-title {
  font-size: 12px !important;
}
</style>
