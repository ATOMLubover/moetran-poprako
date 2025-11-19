<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';

interface SourcePosition {
  x: number;
  y: number;
  width: number;
  height: number;
}

type SourceStatus = 'empty' | 'translated' | 'proofed';

type SourceCategory = 'outside' | 'inside';

interface TranslationSource {
  id: string;
  category: SourceCategory;
  status: SourceStatus;
  translationText: string;
  proofText: string;
  position: SourcePosition;
}

interface ProjectPageData {
  imageUrl: string;
  pageIndex: number;
  pageCount: number;
  title: string;
  sources: TranslationSource[];
}

const props = defineProps<{
  projectId: string;
  pageIndex: number;
}>();

const emit = defineEmits<{
  (event: 'update:pageIndex', value: number): void;
  (event: 'back'): void;
}>();

const MIN_ZOOM = 0.6;

const MAX_ZOOM = 2.4;

const ZOOM_STEP = 0.12;

const PANNING_THRESHOLD = 6;

const DEFAULT_EDITOR_POSITION = { x: 60, y: 24 };

const MOCK_IMAGE_URLS = [
  new URL('../../../tests/images/cover_miseyari_01_0001.jpg', import.meta.url).href,
  new URL('../../../tests/images/miseyari_01_0001.jpg', import.meta.url).href,
  new URL('../../../tests/images/miseyari_02_0001.jpg', import.meta.url).href,
  new URL('../../../tests/images/miseyari_03_0001.jpg', import.meta.url).href,
  new URL('../../../tests/images/miseyari_04_0001.jpg', import.meta.url).href,
];

const currentPageIndex = ref(props.pageIndex);

const projectPage = ref<ProjectPageData | null>(null);

const sources = ref<TranslationSource[]>([]);

const zoom = ref(1);

const pan = ref({ x: 0, y: 0 });

const activeSourceId = ref<string | null>(null);

const isLoading = ref(false);

const editorTranslationText = ref('');

const editorProofText = ref('');

const editorPosition = ref({ ...DEFAULT_EDITOR_POSITION });

const isDraggingEditor = ref(false);

const editorPointerStart = ref({ x: 0, y: 0 });

const editorPositionStart = ref({ ...DEFAULT_EDITOR_POSITION });

const imageWrapperRef = ref<HTMLDivElement | null>(null);

const draggingSourceId = ref<string | null>(null);

const markerDragOffset = ref({ x: 0, y: 0 });

const isCanvasPointerDown = ref(false);

const isPanningCanvas = ref(false);

const canvasPointerId = ref<number | null>(null);

const canvasPointerStart = ref({ x: 0, y: 0 });

const panStart = ref({ x: 0, y: 0 });

const lastCanvasPointerEvent = ref<PointerEvent | null>(null);

let sourceSerial = 0;

const hasEditorBeenDragged = ref(false);

const boardTransform = computed(
  () => `matrix(${zoom.value}, 0, 0, ${zoom.value}, ${pan.value.x}, ${pan.value.y})`
);

const isBoardGrabbing = computed(() => isCanvasPointerDown.value && isPanningCanvas.value);

const selectedSource = computed(() => {
  if (!activeSourceId.value) {
    return null;
  }

  const target = sources.value.find(item => item.id === activeSourceId.value) ?? null;

  return target;
});

const sourceLabelMap = computed(() => {
  const labelMap = new Map<string, string>();

  let outsideIndex = 1;

  let insideIndex = 1;

  for (const item of sources.value) {
    if (item.category === 'outside') {
      labelMap.set(item.id, `框外-${outsideIndex}`);

      outsideIndex += 1;

      continue;
    }

    labelMap.set(item.id, `框内-${insideIndex}`);

    insideIndex += 1;
  }

  return labelMap;
});

const selectedSourceLabel = computed(() => {
  if (!selectedSource.value) {
    return '';
  }

  return sourceLabelMap.value.get(selectedSource.value.id) ?? '';
});

watch(
  () => props.pageIndex,
  value => {
    currentPageIndex.value = value;
  }
);

watch(selectedSource, source => {
  if (!source) {
    editorTranslationText.value = '';

    editorProofText.value = '';

    moveEditorToDefault();

    return;
  }

  editorTranslationText.value = source.translationText;

  editorProofText.value = source.proofText;

  hasEditorBeenDragged.value = false;

  moveEditorNearSource(true);
});

watch(editorTranslationText, value => {
  if (!selectedSource.value) {
    return;
  }

  selectedSource.value.translationText = value;

  if (selectedSource.value.status === 'proofed') {
    return;
  }

  if (value.trim().length === 0) {
    selectedSource.value.status = 'empty';

    return;
  }

  selectedSource.value.status = 'translated';
});

watch(editorProofText, value => {
  if (!selectedSource.value) {
    return;
  }

  selectedSource.value.proofText = value;
});

watch(
  currentPageIndex,
  value => {
    initPage(value);
  },
  { immediate: true }
);

// 获取 mock 页面数据
async function __mockFetchProjectPage(pageIndex: number): Promise<ProjectPageData> {
  sourceSerial = pageIndex * 100;

  const imageIndex = pageIndex % MOCK_IMAGE_URLS.length;

  const baseSources: TranslationSource[] = [
    {
      id: createSourceId(),
      category: 'inside',
      status: 'translated',
      translationText: '这里是翻译示例。',
      proofText: '',
      position: {
        x: 0.18,
        y: 0.18,
        width: 0.32,
        height: 0.12,
      },
    },
    {
      id: createSourceId(),
      category: 'outside',
      status: 'empty',
      translationText: '',
      proofText: '',
      position: {
        x: 0.58,
        y: 0.12,
        width: 0.2,
        height: 0.1,
      },
    },
    {
      id: createSourceId(),
      category: 'inside',
      status: 'proofed',
      translationText: '最终校对文本示例。',
      proofText: '最终校对文本示例。',
      position: {
        x: 0.26,
        y: 0.62,
        width: 0.3,
        height: 0.16,
      },
    },
  ];

  return new Promise(resolve => {
    window.setTimeout(() => {
      resolve({
        imageUrl: MOCK_IMAGE_URLS[imageIndex],
        pageIndex: pageIndex + 1,
        pageCount: 24,
        title: 'MOE 漫画项目',
        sources: baseSources,
      });
    }, 160);
  });
}

// 初始化页面数据
async function initPage(pageIndex: number): Promise<void> {
  if (isLoading.value) {
    return;
  }

  isLoading.value = true;

  try {
    const result = await __mockFetchProjectPage(pageIndex);

    projectPage.value = result;

    sources.value = [...result.sources];

    activeSourceId.value = result.sources[0]?.id ?? null;
  } catch (error) {
    console.error('加载项目页面失败', error);
  } finally {
    editorTranslationText.value = selectedSource.value?.translationText ?? '';

    editorProofText.value = selectedSource.value?.proofText ?? '';

    zoom.value = 1;

    pan.value = { x: 0, y: 0 };

    isLoading.value = false;
  }
}

// 生成唯一 ID
function createSourceId(): string {
  sourceSerial += 1;

  return `${props.projectId}-src-${sourceSerial}`;
}

// 通过坐标创建标记
function addSourceAtPointer(event: MouseEvent | PointerEvent): void {
  const wrapper = imageWrapperRef.value;

  if (!wrapper || !projectPage.value) {
    return;
  }

  const rect = wrapper.getBoundingClientRect();

  const relativeX = (event.clientX - rect.left) / rect.width;

  const relativeY = (event.clientY - rect.top) / rect.height;

  if (relativeX < 0 || relativeX > 1 || relativeY < 0 || relativeY > 1) {
    return;
  }

  const newSource: TranslationSource = {
    id: createSourceId(),
    category: 'inside',
    status: 'empty',
    translationText: '',
    proofText: '',
    position: {
      x: Math.min(Math.max(relativeX - 0.12, 0), 0.88),
      y: Math.min(Math.max(relativeY - 0.08, 0), 0.88),
      width: 0.24,
      height: 0.16,
    },
  };

  sources.value = [...sources.value, newSource];

  activeSourceId.value = newSource.id;
}

// 拖拽标记开始
function handleSourcePointerDown(event: PointerEvent, sourceId: string): void {
  if (event.button !== 0) {
    return;
  }

  event.stopPropagation();

  const wrapper = imageWrapperRef.value;

  const targetSource = sources.value.find(item => item.id === sourceId);

  if (!wrapper || !targetSource) {
    return;
  }

  const rect = wrapper.getBoundingClientRect();

  const offsetX = event.clientX - (targetSource.position.x * rect.width + rect.left);

  const offsetY = event.clientY - (targetSource.position.y * rect.height + rect.top);

  markerDragOffset.value = {
    x: offsetX,
    y: offsetY,
  };

  draggingSourceId.value = sourceId;

  activeSourceId.value = sourceId;
}

// 拖拽标记移动
function updateDraggingSourcePosition(event: PointerEvent): void {
  if (!draggingSourceId.value) {
    return;
  }

  const wrapper = imageWrapperRef.value;

  const targetSource = sources.value.find(item => item.id === draggingSourceId.value);

  if (!wrapper || !targetSource) {
    return;
  }

  const rect = wrapper.getBoundingClientRect();

  const newLeft = event.clientX - markerDragOffset.value.x - rect.left;

  const newTop = event.clientY - markerDragOffset.value.y - rect.top;

  const relativeX = newLeft / rect.width;

  const relativeY = newTop / rect.height;

  targetSource.position.x = Math.min(Math.max(relativeX, 0), 1 - targetSource.position.width);

  targetSource.position.y = Math.min(Math.max(relativeY, 0), 1 - targetSource.position.height);
}

// 停止拖拽标记
function stopDraggingSource(): void {
  if (!draggingSourceId.value) {
    return;
  }

  draggingSourceId.value = null;
}

// 删除标记
function handleRemoveSource(sourceId: string): void {
  sources.value = sources.value.filter(item => item.id !== sourceId);

  if (activeSourceId.value === sourceId) {
    activeSourceId.value = sources.value[0]?.id ?? null;
  }
}

// 更新缩放
function handleCanvasWheel(event: WheelEvent): void {
  if (!projectPage.value) {
    return;
  }

  event.preventDefault();

  const wrapper = imageWrapperRef.value;

  if (!wrapper) {
    return;
  }

  const direction = event.deltaY > 0 ? -1 : 1;

  const next = Number((zoom.value + direction * ZOOM_STEP).toFixed(2));

  const nextZoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, next));

  if (nextZoom === zoom.value) {
    return;
  }

  const rect = wrapper.getBoundingClientRect();

  const baseLeft = rect.left - pan.value.x;

  const baseTop = rect.top - pan.value.y;

  const offsetX = event.clientX - rect.left;

  const offsetY = event.clientY - rect.top;

  const baseX = offsetX / zoom.value;

  const baseY = offsetY / zoom.value;

  const nextPanX = event.clientX - baseLeft - nextZoom * baseX;

  const nextPanY = event.clientY - baseTop - nextZoom * baseY;

  pan.value = {
    x: nextPanX,
    y: nextPanY,
  };

  zoom.value = nextZoom;
}

// 处理画布鼠标按下
function handleCanvasPointerDown(event: PointerEvent): void {
  if (event.button !== 0) {
    return;
  }

  const target = event.target as HTMLElement;

  if (target.closest('[data-source-id]')) {
    return;
  }

  isCanvasPointerDown.value = true;

  isPanningCanvas.value = false;

  canvasPointerId.value = event.pointerId;

  canvasPointerStart.value = {
    x: event.clientX,
    y: event.clientY,
  };

  panStart.value = { ...pan.value };

  lastCanvasPointerEvent.value = event;

  const currentTarget = event.currentTarget as HTMLElement | null;

  currentTarget?.setPointerCapture?.(event.pointerId);
}

function handleCanvasPointerMove(event: PointerEvent): void {
  if (!isCanvasPointerDown.value || canvasPointerId.value !== event.pointerId) {
    return;
  }

  lastCanvasPointerEvent.value = event;

  const deltaX = event.clientX - canvasPointerStart.value.x;

  const deltaY = event.clientY - canvasPointerStart.value.y;

  if (!isPanningCanvas.value) {
    const distance = Math.hypot(deltaX, deltaY);

    if (distance >= PANNING_THRESHOLD) {
      isPanningCanvas.value = true;
    }
  }

  if (!isPanningCanvas.value) {
    return;
  }

  pan.value = {
    x: panStart.value.x + deltaX,
    y: panStart.value.y + deltaY,
  };
}

function handleCanvasPointerUp(event: PointerEvent): void {
  if (!isCanvasPointerDown.value || canvasPointerId.value !== event.pointerId) {
    return;
  }

  const currentTarget = event.currentTarget as HTMLElement | null;

  currentTarget?.releasePointerCapture?.(event.pointerId);

  const pointerWasPanning = isPanningCanvas.value;

  isCanvasPointerDown.value = false;

  isPanningCanvas.value = false;

  canvasPointerId.value = null;

  const referenceEvent = lastCanvasPointerEvent.value ?? event;

  lastCanvasPointerEvent.value = null;

  if (pointerWasPanning) {
    return;
  }

  const wrapper = imageWrapperRef.value;

  if (!wrapper) {
    return;
  }

  const rect = wrapper.getBoundingClientRect();

  if (
    referenceEvent.clientX < rect.left ||
    referenceEvent.clientX > rect.right ||
    referenceEvent.clientY < rect.top ||
    referenceEvent.clientY > rect.bottom
  ) {
    return;
  }

  addSourceAtPointer(referenceEvent);
}

function handleCanvasPointerCancel(event: PointerEvent): void {
  if (!isCanvasPointerDown.value || canvasPointerId.value !== event.pointerId) {
    return;
  }

  const currentTarget = event.currentTarget as HTMLElement | null;

  currentTarget?.releasePointerCapture?.(event.pointerId);

  isCanvasPointerDown.value = false;

  isPanningCanvas.value = false;

  canvasPointerId.value = null;

  lastCanvasPointerEvent.value = null;
}

// 选中标记
function handleSelectSource(sourceId: string): void {
  if (activeSourceId.value === sourceId) {
    return;
  }

  activeSourceId.value = sourceId;
}

// 拖拽编辑器开始
function handleEditorPointerDown(event: PointerEvent): void {
  event.preventDefault();

  isDraggingEditor.value = true;

  editorPointerStart.value = {
    x: event.clientX,
    y: event.clientY,
  };

  editorPositionStart.value = { ...editorPosition.value };
}

// 拖拽编辑器移动
function updateEditorPosition(event: PointerEvent): void {
  if (!isDraggingEditor.value) {
    return;
  }

  const deltaX = event.clientX - editorPointerStart.value.x;

  const deltaY = event.clientY - editorPointerStart.value.y;

  const nextX = editorPositionStart.value.x + (deltaX / window.innerWidth) * 100;

  const nextY = editorPositionStart.value.y + (deltaY / window.innerHeight) * 100;

  editorPosition.value = {
    x: Math.min(92, Math.max(6, nextX)),
    y: Math.min(90, Math.max(8, nextY)),
  };

  if (!hasEditorBeenDragged.value && (Math.abs(deltaX) > 1 || Math.abs(deltaY) > 1)) {
    hasEditorBeenDragged.value = true;
  }
}

// 拖拽编辑器结束
function stopDraggingEditor(): void {
  if (!isDraggingEditor.value) {
    return;
  }

  isDraggingEditor.value = false;
}

// 移动编辑器到默认位置
function moveEditorToDefault(): void {
  editorPosition.value = { ...DEFAULT_EDITOR_POSITION };

  hasEditorBeenDragged.value = false;
}

// 根据标记位置移动编辑器
function moveEditorNearSource(force = false): void {
  if (hasEditorBeenDragged.value && !force) {
    return;
  }

  const wrapper = imageWrapperRef.value;

  if (!wrapper || !selectedSource.value) {
    moveEditorToDefault();

    return;
  }

  const rect = wrapper.getBoundingClientRect();

  const source = selectedSource.value;

  const left = rect.left + source.position.x * rect.width + source.position.width * rect.width;

  const top = rect.top + source.position.y * rect.height;

  const nextX = ((left + 24) / window.innerWidth) * 100;

  const nextY = (top / window.innerHeight) * 100;

  editorPosition.value = {
    x: Math.min(92, Math.max(6, nextX)),
    y: Math.min(88, Math.max(9, nextY)),
  };
}

// 切换标记类别
function toggleSourceCategory(): void {
  if (!selectedSource.value) {
    return;
  }

  selectedSource.value.category = selectedSource.value.category === 'inside' ? 'outside' : 'inside';
}

// 切换校对状态
function toggleProofStatus(): void {
  if (!selectedSource.value) {
    return;
  }

  if (selectedSource.value.status === 'proofed') {
    selectedSource.value.status =
      selectedSource.value.translationText.trim().length === 0 ? 'empty' : 'translated';

    selectedSource.value.proofText = '';

    editorProofText.value = '';

    return;
  }

  const proofCandidate = editorProofText.value.trim() || editorTranslationText.value.trim();

  if (proofCandidate.length === 0) {
    return;
  }

  selectedSource.value.translationText = proofCandidate;

  selectedSource.value.proofText = proofCandidate;

  editorTranslationText.value = proofCandidate;

  editorProofText.value = proofCandidate;

  selectedSource.value.status = 'proofed';
}

// 关闭悬浮窗
function handleCloseEditor(): void {
  activeSourceId.value = null;
}

// 返回项目详情
function handleBackToProject(): void {
  emit('back');
}

// 切换上一页
function handlePreviousPage(): void {
  if (currentPageIndex.value <= 0) {
    return;
  }

  currentPageIndex.value -= 1;

  emit('update:pageIndex', currentPageIndex.value);
}

// 切换下一页
function handleNextPage(): void {
  const total = projectPage.value?.pageCount ?? 0;

  if (total === 0 || currentPageIndex.value >= total - 1) {
    return;
  }

  currentPageIndex.value += 1;

  emit('update:pageIndex', currentPageIndex.value);
}

onMounted(() => {
  window.addEventListener('pointermove', updateDraggingSourcePosition);

  window.addEventListener('pointermove', updateEditorPosition);

  window.addEventListener('pointerup', stopDraggingSource);

  window.addEventListener('pointerup', stopDraggingEditor);

  window.addEventListener('pointercancel', stopDraggingSource);

  window.addEventListener('pointercancel', stopDraggingEditor);
});

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', updateDraggingSourcePosition);

  window.removeEventListener('pointermove', updateEditorPosition);

  window.removeEventListener('pointerup', stopDraggingSource);

  window.removeEventListener('pointerup', stopDraggingEditor);

  window.removeEventListener('pointercancel', stopDraggingSource);

  window.removeEventListener('pointercancel', stopDraggingEditor);
});
</script>

<template>
  <section class="translator">
    <header class="translator__header">
      <div class="translator__title-group">
        <span class="translator__badge"
          >第 {{ projectPage?.pageIndex ?? '-' }} / {{ projectPage?.pageCount ?? '-' }} 页</span
        >
        <h1 class="translator__title">{{ projectPage?.title ?? '加载中' }}</h1>
      </div>
      <div class="translator__zoom-indicator">{{ Math.round(zoom * 100) }}%</div>
    </header>
    <div class="translator__body">
      <aside class="toolbox">
        <button type="button" class="toolbox__button" title="返回项目" @click="handleBackToProject">
          <span class="sr-only">返回项目</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 5L4 12H7V20H11V15H13V20H17V12H20L12 5Z" />
          </svg>
        </button>
        <button
          type="button"
          class="toolbox__button"
          title="上一页"
          @click="handlePreviousPage"
          :disabled="currentPageIndex <= 0"
        >
          <span class="sr-only">上一页</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M14.5 6L8.5 12L14.5 18" />
          </svg>
        </button>
        <button
          type="button"
          class="toolbox__button"
          title="下一页"
          @click="handleNextPage"
          :disabled="projectPage ? currentPageIndex >= projectPage.pageCount - 1 : true"
        >
          <span class="sr-only">下一页</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M9.5 6L15.5 12L9.5 18" />
          </svg>
        </button>
      </aside>
      <div class="translator__workspace">
        <div
          class="board"
          :class="{ 'board--grabbing': isBoardGrabbing }"
          @wheel="handleCanvasWheel"
          @pointerdown="handleCanvasPointerDown"
          @pointermove="handleCanvasPointerMove"
          @pointerup="handleCanvasPointerUp"
          @pointercancel="handleCanvasPointerCancel"
          @contextmenu.prevent
        >
          <div class="board__frame" ref="imageWrapperRef" :style="{ transform: boardTransform }">
            <img
              v-if="projectPage"
              :src="projectPage.imageUrl"
              alt="漫画页"
              class="board__image"
              draggable="false"
            />
            <button
              v-for="item in sources"
              :key="item.id"
              type="button"
              class="marker"
              :class="{
                'marker--active': item.id === activeSourceId,
                'marker--translated': item.status === 'translated',
                'marker--proofed': item.status === 'proofed',
                'marker--outside': item.category === 'outside',
                'marker--inside': item.category === 'inside',
              }"
              :data-source-id="item.id"
              :style="{
                left: `${item.position.x * 100}%`,
                top: `${item.position.y * 100}%`,
                width: `${item.position.width * 100}%`,
                height: `${item.position.height * 100}%`,
              }"
              @pointerdown="event => handleSourcePointerDown(event, item.id)"
              @click.stop="handleSelectSource(item.id)"
              @contextmenu.prevent.stop="handleRemoveSource(item.id)"
            >
              <span class="marker__label">{{ sourceLabelMap.get(item.id) }}</span>
              <span class="marker__pointer" aria-hidden="true"></span>
            </button>
          </div>
        </div>
        <aside class="panel">
          <ul class="panel__list">
            <li
              v-for="item in sources"
              :key="item.id"
              class="panel__item"
              :class="{
                'panel__item--active': item.id === activeSourceId,
                'panel__item--empty': item.status === 'empty',
                'panel__item--proofed': item.status === 'proofed',
                'panel__item--translated': item.status === 'translated',
              }"
              @click="handleSelectSource(item.id)"
            >
              <div class="panel__item-top">
                <span>{{ sourceLabelMap.get(item.id) }}</span>
                <span
                  class="panel__status-dot"
                  :class="{
                    'panel__status-dot--empty': item.status === 'empty',
                    'panel__status-dot--translated': item.status === 'translated',
                    'panel__status-dot--proofed': item.status === 'proofed',
                  }"
                  aria-hidden="true"
                ></span>
              </div>
              <p class="panel__item-text">
                {{
                  item.status === 'proofed' ? item.proofText || '—' : item.translationText || '—'
                }}
              </p>
            </li>
          </ul>
        </aside>
      </div>
    </div>
    <div
      v-if="selectedSource"
      class="editor"
      :style="{
        left: `${editorPosition.x}vw`,
        top: `${editorPosition.y}vh`,
      }"
    >
      <header class="editor__header" @pointerdown="handleEditorPointerDown">
        <div class="editor__title">
          <span class="editor__label">{{ selectedSourceLabel }}</span>
          <button type="button" class="editor__toggle" @click.stop="toggleSourceCategory">
            {{ selectedSource.category === 'inside' ? '框内' : '框外' }}
          </button>
        </div>
        <button
          type="button"
          class="editor__close"
          title="关闭浮窗"
          @click.stop="handleCloseEditor"
          @pointerdown.stop
        >
          <span class="sr-only">关闭浮窗</span>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M8 8L16 16M16 8L8 16" />
          </svg>
        </button>
      </header>
      <div class="editor__body">
        <textarea
          v-model="editorTranslationText"
          class="editor__textarea"
          placeholder="翻译文本"
        ></textarea>
        <textarea
          v-model="editorProofText"
          class="editor__textarea editor__textarea--proof"
          placeholder="校对文本"
        ></textarea>
      </div>
      <footer class="editor__footer">
        <button type="button" class="editor__action" @click="toggleProofStatus">
          {{ selectedSource.status === 'proofed' ? '取消校对状态' : '标记为已校对' }}
        </button>
      </footer>
    </div>
  </section>
</template>

<style scoped>
.translator {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: radial-gradient(circle at 18% 22%, #f6fbff 0%, #f1f6ff 45%, #ffffff 100%);
  color: #28384c;
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
}

.translator__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 22px 40px 16px;
}

.translator__title-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.translator__badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 999px;
  background: rgba(112, 182, 255, 0.2);
  color: #3d6696;
  font-size: 12px;
  letter-spacing: 0.3px;
}

.translator__title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: #1f2e43;
}

.translator__zoom-indicator {
  padding: 6px 14px;
  border-radius: 999px;
  background: rgba(130, 190, 255, 0.18);
  color: #2f5a8f;
  font-size: 13px;
}

.translator__body {
  flex: 1;
  display: flex;
  gap: 18px;
  padding: 0 36px 40px;
  align-items: stretch;
  min-height: 0;
}

.toolbox {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px 10px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.8);
  box-shadow: 0 18px 36px rgba(129, 167, 215, 0.16);
  align-self: flex-start;
}

.toolbox__button {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  border: 1px solid rgba(152, 186, 229, 0.45);
  background: rgba(241, 248, 255, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease,
    border 0.18s ease;
}

.toolbox__button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 22px rgba(136, 190, 247, 0.28);
  border-color: rgba(112, 182, 255, 0.6);
}

.toolbox__button:disabled {
  opacity: 0.35;
  cursor: not-allowed;
  box-shadow: none;
}

.toolbox__icon {
  width: 18px;
  height: 18px;
  stroke: #3e5c88;
  fill: none;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.translator__workspace {
  flex: 1;
  display: flex;
  gap: 18px;
  align-items: stretch;
  min-height: 0;
}

.board {
  position: relative;
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 26px;
  background: rgba(255, 255, 255, 0.85);
  box-shadow: 0 26px 60px rgba(118, 166, 219, 0.18);
  backdrop-filter: blur(10px);
  overflow: hidden;
  cursor: grab;
  touch-action: none;
  min-height: 0;
  height: 100%;
}

.board--grabbing {
  cursor: grabbing;
}

.board__frame {
  position: relative;
  width: clamp(360px, 58vw, 680px);
  transform-origin: top left;
  transition: transform 0.08s ease;
}

.board__image {
  width: 100%;
  height: auto;
  display: block;
  border-radius: 16px;
  box-shadow: 0 16px 32px rgba(113, 156, 212, 0.22);
  user-select: none;
}

.marker {
  position: absolute;
  border: none;
  background: transparent;
  cursor: grab;
  padding: 0;
}

.marker:focus-visible {
  outline: 2px solid rgba(118, 184, 255, 0.7);
  outline-offset: 4px;
}

.marker__label {
  position: absolute;
  top: -28px;
  left: 50%;
  transform: translateX(-50%);
  padding: 4px 12px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.95);
  color: #35506f;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.4px;
  box-shadow: 0 8px 16px rgba(94, 138, 196, 0.18);
  pointer-events: none;
}

.marker__pointer {
  position: absolute;
  left: 50%;
  bottom: -20px;
  width: 22px;
  height: 18px;
  transform: translateX(-50%);
  background: rgba(120, 185, 255, 0.6);
  clip-path: polygon(50% 100%, 0% 0%, 100% 0%);
  filter: drop-shadow(0 6px 10px rgba(94, 138, 196, 0.22));
  pointer-events: none;
}

.marker--active .marker__label {
  background: rgba(255, 255, 255, 0.98);
  color: #f07429;
  box-shadow: 0 10px 24px rgba(255, 188, 140, 0.3);
}

.marker--active .marker__pointer {
  background: rgba(255, 188, 140, 0.78);
}

.marker--translated .marker__label {
  background: rgba(255, 255, 255, 0.96);
  color: #2f6c4d;
  box-shadow: 0 10px 24px rgba(124, 207, 171, 0.26);
}

.marker--translated .marker__pointer {
  background: rgba(124, 207, 171, 0.62);
}

.marker--proofed .marker__label {
  background: rgba(255, 255, 255, 0.98);
  color: #4b3ab8;
  box-shadow: 0 10px 24px rgba(151, 146, 255, 0.3);
}

.marker--proofed .marker__pointer {
  background: rgba(151, 146, 255, 0.68);
}

.panel {
  width: 320px;
  flex-shrink: 0;
  padding: 18px 18px 16px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.88);
  box-shadow: 0 24px 50px rgba(123, 168, 225, 0.18);
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.panel__list {
  margin: 0;
  padding: 8px 6px 4px 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
}

.panel__item {
  flex: none;
  width: 100%;
  padding: 11px 13px;
  border-radius: 15px;
  background: rgba(235, 243, 255, 0.8);
  border: 1px solid rgba(170, 204, 245, 0.42);
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    border 0.16s ease;
}

.panel__item--active {
  border-color: rgba(118, 184, 255, 0.85);
  box-shadow: 0 12px 26px rgba(150, 189, 246, 0.26);
  transform: translateY(-2px);
}

.panel__item--empty {
  background: rgba(255, 240, 240, 0.7);
  border-color: rgba(255, 210, 210, 0.55);
}

.panel__item--proofed {
  background: rgba(235, 232, 255, 0.75);
  border-color: rgba(200, 194, 255, 0.6);
}

.panel__item--translated {
  background: rgba(222, 246, 235, 0.7);
  border-color: rgba(150, 210, 185, 0.6);
}

.panel__item-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: #3a4d69;
  margin-bottom: 6px;
}

.panel__status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(120, 185, 255, 0.6);
  box-shadow: 0 0 0 3px rgba(120, 185, 255, 0.16);
}

.panel__status-dot--empty {
  background: rgba(255, 160, 160, 0.75);
  box-shadow: 0 0 0 3px rgba(255, 160, 160, 0.18);
}

.panel__status-dot--translated {
  background: rgba(124, 207, 171, 0.8);
  box-shadow: 0 0 0 3px rgba(124, 207, 171, 0.2);
}

.panel__status-dot--proofed {
  background: rgba(151, 146, 255, 0.85);
  box-shadow: 0 0 0 3px rgba(151, 146, 255, 0.22);
}

.panel__item-text {
  margin: 0;
  font-size: 13px;
  color: #4b5e7a;
  line-height: 1.5;
  max-height: 3.4em;
  overflow: hidden;
}

.editor {
  position: fixed;
  width: 280px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.97);
  box-shadow: 0 22px 48px rgba(120, 162, 218, 0.26);
  border: 1px solid rgba(162, 192, 233, 0.4);
  backdrop-filter: blur(10px);
  z-index: 30;
  overflow: hidden;
}

.editor__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 10px;
  cursor: grab;
  background: rgba(248, 252, 255, 0.95);
  border-bottom: 1px solid rgba(190, 210, 238, 0.55);
}

.editor__title {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.editor__label {
  font-weight: 600;
  font-size: 14px;
  color: #294061;
}

.editor__toggle {
  align-self: flex-start;
  border: 1px solid rgba(170, 204, 245, 0.7);
  background: rgba(236, 244, 255, 0.8);
  border-radius: 999px;
  padding: 4px 12px;
  color: #325072;
  font-size: 12px;
  cursor: pointer;
  transition:
    background 0.16s ease,
    box-shadow 0.16s ease;
}

.editor__toggle:hover {
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 8px 18px rgba(150, 190, 240, 0.28);
}

.editor__close {
  border: none;
  background: transparent;
  padding: 6px;
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.16s ease;
}

.editor__close:hover {
  background: rgba(230, 240, 255, 0.8);
}

.editor__close svg {
  width: 16px;
  height: 16px;
  stroke: #3b5a80;
  fill: none;
  stroke-width: 1.8;
  stroke-linecap: round;
}

.editor__body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
}

.editor__textarea {
  width: 100%;
  min-height: 108px;
  border: 1px solid rgba(188, 206, 233, 0.85);
  border-radius: 14px;
  padding: 11px;
  font-size: 13px;
  line-height: 1.6;
  color: #2a3b4f;
  background: rgba(246, 250, 255, 0.92);
  resize: vertical;
  transition:
    border 0.16s ease,
    box-shadow 0.16s ease;
  box-sizing: border-box;
}

.editor__textarea:focus {
  outline: none;
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 0 0 3px rgba(118, 184, 255, 0.18);
}

.editor__textarea--proof {
  min-height: 96px;
  background: rgba(244, 240, 255, 0.94);
}

.editor__footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 14px 14px;
}

.editor__action {
  min-width: 160px;
  border: none;
  border-radius: 999px;
  padding: 8px 0;
  background: linear-gradient(120deg, rgba(124, 205, 182, 0.92), rgba(146, 214, 222, 0.9));
  color: #234060;
  font-size: 13px;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease;
}

.editor__action:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 20px rgba(130, 205, 182, 0.26);
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

@media (max-width: 1280px) {
  .translator__body {
    flex-direction: column;
    align-items: stretch;
  }

  .toolbox {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
  }

  .translator__workspace {
    flex-direction: column;
  }

  .panel {
    width: 100%;
    height: auto;
  }

  .panel__list {
    flex-direction: row;
    flex-wrap: wrap;
    max-height: none;
    gap: 10px;
    padding-right: 6px;
  }

  .panel__item {
    flex: 1 1 220px;
    width: auto;
  }
}
</style>
