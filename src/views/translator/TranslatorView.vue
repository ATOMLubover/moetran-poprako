<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

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

interface SourceLabelInfo {
  number: string;
  display: string;
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

const DEFAULT_EDITOR_POSITION = { x: 72, y: 72 };

const DEFAULT_EDITOR_SIZE = { width: 320, height: 280 };

const MIN_WORKSPACE_HEIGHT = 420;

const WORKSPACE_VERTICAL_OFFSET = 220;

const PANEL_COLLAPSE_BREAKPOINT = 1180;

const PANEL_COLLAPSED_WIDTH = 56;

const PANEL_WIDTH_RATIO = 0.2;

const MARKER_REFERENCE_WIDTH = 1280;

const MIN_MARKER_VIEWPORT_SCALE = 0.9;

const MAX_MARKER_VIEWPORT_SCALE = 1.2;

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

const pageInputValue = ref(currentPageIndex.value + 1);

const zoom = ref(1);

const pan = ref({ x: 0, y: 0 });

const activeSourceId = ref<string | null>(null);

const isLoading = ref(false);

const editorTranslationText = ref('');

const editorProofText = ref('');

const translationTextareaRef = ref<HTMLTextAreaElement | null>(null);

const editorPosition = ref({ ...DEFAULT_EDITOR_POSITION });

const editorSize = ref({ ...DEFAULT_EDITOR_SIZE });

const isDraggingEditor = ref(false);

const editorPointerStart = ref({ x: 0, y: 0 });

const editorPositionStart = ref({ ...DEFAULT_EDITOR_POSITION });

const imageWrapperRef = ref<HTMLDivElement | null>(null);

const panelRef = ref<HTMLElement | null>(null);

const windowSize = ref({ width: window.innerWidth, height: window.innerHeight });

const hasPanelOverride = ref(false);

const isPanelCollapsed = ref(window.innerWidth < PANEL_COLLAPSE_BREAKPOINT);

const draggingSourceId = ref<string | null>(null);

const markerDragOffset = ref({ x: 0, y: 0 });

const isCanvasPointerDown = ref(false);

const isPanningCanvas = ref(false);

const canvasPointerId = ref<number | null>(null);

const canvasPointerStart = ref({ x: 0, y: 0 });

const panStart = ref({ x: 0, y: 0 });

const lastCanvasPointerEvent = ref<PointerEvent | null>(null);

const canvasPointerButton = ref<0 | 2 | null>(null);

let sourceSerial = 0;

let latestPageLoadToken = 0;

const pageSourceStore = new Map<number, TranslationSource[]>();

const hasEditorBeenDragged = ref(false);

function cloneSource(source: TranslationSource): TranslationSource {
  return {
    ...source,
    position: { ...source.position },
  };
}

function cloneSourceList(list: TranslationSource[]): TranslationSource[] {
  return list.map(item => cloneSource(item));
}

function persistPageSources(pageIndex: number): void {
  if (pageIndex < 0) {
    return;
  }

  pageSourceStore.set(pageIndex, cloneSourceList(sources.value));
}

const boardTransform = computed(
  () => `matrix(${zoom.value}, 0, 0, ${zoom.value}, ${pan.value.x}, ${pan.value.y})`
);

const showMarkers = computed(() => !isPanelCollapsed.value);

const workspaceStyle = computed(() => {
  const availableHeight = Math.max(
    MIN_WORKSPACE_HEIGHT,
    windowSize.value.height - WORKSPACE_VERTICAL_OFFSET
  );

  const style: Record<string, string> = {
    minHeight: `${availableHeight}px`,
    height: `${availableHeight}px`,
  };

  if (isPanelCollapsed.value) {
    style.gridTemplateColumns = `minmax(0, 1fr) ${PANEL_COLLAPSED_WIDTH}px`;

    return style;
  }

  const panelPercent = Math.round(PANEL_WIDTH_RATIO * 100);

  const boardPercent = Math.max(0, 100 - panelPercent);

  style.gridTemplateColumns = `minmax(0, ${boardPercent}%) minmax(260px, ${panelPercent}%)`;

  return style;
});

const boardGridClass = computed(() => ({ 'board--full': isPanelCollapsed.value }));

const markerViewportScale = computed(() => {
  const widthFactor = windowSize.value.width / MARKER_REFERENCE_WIDTH;

  return Math.min(MAX_MARKER_VIEWPORT_SCALE, Math.max(MIN_MARKER_VIEWPORT_SCALE, widthFactor));
});

const markerScale = computed(() => markerViewportScale.value / zoom.value);

const markerTransform = computed(() => `translate(-50%, -100%) scale(${markerScale.value})`);

const selectedSource = computed(() => {
  if (!activeSourceId.value) {
    return null;
  }

  const target = sources.value.find(item => item.id === activeSourceId.value) ?? null;

  return target;
});

const sourceLabelMap = computed(() => {
  const labelMap = new Map<string, SourceLabelInfo>();

  let index = 1;

  for (const item of sources.value) {
    const number = String(index).padStart(3, '0');

    const suffix = item.category === 'inside' ? '框内' : '框外';

    labelMap.set(item.id, {
      number,
      display: `${number}-${suffix}`,
    });

    index += 1;
  }

  return labelMap;
});

const selectedSourceLabel = computed(() => {
  if (!selectedSource.value) {
    return '';
  }

  return sourceLabelMap.value.get(selectedSource.value.id)?.display ?? '';
});

watch(
  () => props.pageIndex,
  value => {
    currentPageIndex.value = value;
  }
);

watch(selectedSource, (source, previousSource) => {
  if (!source) {
    editorTranslationText.value = '';

    editorProofText.value = '';

    moveEditorToDefault();

    return;
  }

  editorTranslationText.value = source.translationText;

  editorProofText.value = source.proofText;

  if (!previousSource) {
    hasEditorBeenDragged.value = false;

    moveEditorNearPanel(true);
  }

  nextTick(() => {
    translationTextareaRef.value?.focus();
  });
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
  (value, oldValue) => {
    pageInputValue.value = value + 1;

    if (typeof oldValue === 'number') {
      persistPageSources(oldValue);
    }

    initPage(value);
  },
  { immediate: true }
);

watch(windowSize, value => {
  clampEditorWithinViewport();

  if (!hasPanelOverride.value) {
    isPanelCollapsed.value = value.width < PANEL_COLLAPSE_BREAKPOINT;
  }
});

// 获取 mock 页面数据
async function __mockFetchProjectPage(pageIndex: number): Promise<ProjectPageData> {
  sourceSerial = pageIndex * 100;

  const imageIndex = pageIndex % MOCK_IMAGE_URLS.length;

  const offset = (pageIndex % 4) * 0.04;

  const verticalShift = (pageIndex % 3) * 0.05;

  const baseSources: TranslationSource[] = [
    {
      id: createSourceId(),
      category: 'inside',
      status: pageIndex % 2 === 0 ? 'translated' : 'empty',
      translationText: `这里是第 ${pageIndex + 1} 页的翻译示例。`,
      proofText: '',
      position: {
        x: 0.22 + offset,
        y: 0.26 + verticalShift,
        width: 0,
        height: 0,
      },
    },
    {
      id: createSourceId(),
      category: 'outside',
      status: 'empty',
      translationText: '',
      proofText: '',
      position: {
        x: 0.55 + offset * 0.8,
        y: 0.2 + verticalShift * 0.6,
        width: 0,
        height: 0,
      },
    },
    {
      id: createSourceId(),
      category: 'inside',
      status: pageIndex % 3 === 0 ? 'proofed' : 'translated',
      translationText: `最终校对文本示例 - 第 ${pageIndex + 1} 页`,
      proofText: pageIndex % 3 === 0 ? `最终校对文本示例 - 第 ${pageIndex + 1} 页` : '',
      position: {
        x: 0.38 + offset * 0.5,
        y: 0.62 + verticalShift,
        width: 0,
        height: 0,
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
  const requestToken = ++latestPageLoadToken;

  isLoading.value = true;

  try {
    const result = await __mockFetchProjectPage(pageIndex);

    if (requestToken !== latestPageLoadToken) {
      return;
    }

    projectPage.value = result;

    if (!pageSourceStore.has(pageIndex)) {
      pageSourceStore.set(pageIndex, cloneSourceList(result.sources));
    }

    const hydratedSources = pageSourceStore.get(pageIndex) ?? [];

    sources.value = cloneSourceList(hydratedSources);

    activeSourceId.value = null;
  } catch (error) {
    console.error('加载项目页面失败', error);
  } finally {
    if (requestToken === latestPageLoadToken) {
      editorTranslationText.value = selectedSource.value?.translationText ?? '';

      editorProofText.value = selectedSource.value?.proofText ?? '';

      zoom.value = 1;

      pan.value = { x: 0, y: 0 };

      isLoading.value = false;
    }
  }
}

// 生成唯一 ID
function createSourceId(): string {
  sourceSerial += 1;

  return `${props.projectId}-src-${sourceSerial}`;
}

// 通过坐标创建标记
function addSourceAtPointer(event: MouseEvent | PointerEvent, category: SourceCategory): void {
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
    category,
    status: 'empty',
    translationText: '',
    proofText: '',
    position: {
      x: Math.min(Math.max(relativeX, 0), 1),
      y: Math.min(Math.max(relativeY, 0), 1),
      width: 0,
      height: 0,
    },
  };

  sources.value = [...sources.value, newSource];

  activeSourceId.value = newSource.id;
}

function handleTogglePanel(): void {
  if (!hasPanelOverride.value) {
    hasPanelOverride.value = true;
  }

  isPanelCollapsed.value = !isPanelCollapsed.value;

  if (isPanelCollapsed.value) {
    activeSourceId.value = null;

    return;
  }

  nextTick(() => {
    moveEditorNearPanel(true);
  });
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

  const pointerX = rect.left + targetSource.position.x * rect.width;

  const pointerY = rect.top + targetSource.position.y * rect.height;

  const offsetX = event.clientX - pointerX;

  const offsetY = event.clientY - pointerY;

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

  const pointerX = event.clientX - markerDragOffset.value.x;

  const pointerY = event.clientY - markerDragOffset.value.y;

  const relativeX = (pointerX - rect.left) / rect.width;

  const relativeY = (pointerY - rect.top) / rect.height;

  targetSource.position.x = Math.min(Math.max(relativeX, 0), 1);

  targetSource.position.y = Math.min(Math.max(relativeY, 0), 1);
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
  if (event.button !== 0 && event.button !== 2) {
    return;
  }

  const target = event.target as HTMLElement;

  if (target.closest('[data-source-id]')) {
    return;
  }

  canvasPointerButton.value = event.button as 0 | 2;

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

  const pointerButton = canvasPointerButton.value;

  canvasPointerButton.value = null;

  if (pointerWasPanning) {
    return;
  }

  if (isPanelCollapsed.value || (pointerButton !== 0 && pointerButton !== 2)) {
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

  const category: SourceCategory = pointerButton === 2 ? 'outside' : 'inside';

  addSourceAtPointer(referenceEvent, category);
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

  canvasPointerButton.value = null;

  lastCanvasPointerEvent.value = null;
}

function handleWindowResize(): void {
  windowSize.value = {
    width: window.innerWidth,
    height: window.innerHeight,
  };
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

  const nextX = editorPositionStart.value.x + deltaX;

  const nextY = editorPositionStart.value.y + deltaY;

  const maxX = window.innerWidth - editorSize.value.width - 12;

  const maxY = window.innerHeight - editorSize.value.height - 12;

  editorPosition.value = {
    x: Math.min(Math.max(12, nextX), Math.max(12, maxX)),
    y: Math.min(Math.max(12, nextY), Math.max(12, maxY)),
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

  editorSize.value = { ...DEFAULT_EDITOR_SIZE };

  clampEditorWithinViewport();

  hasEditorBeenDragged.value = false;
}

// 保证浮窗在视口内
function clampEditorWithinViewport(): void {
  const maxX = Math.max(12, window.innerWidth - editorSize.value.width - 12);

  const maxY = Math.max(12, window.innerHeight - editorSize.value.height - 12);

  editorPosition.value = {
    x: Math.min(Math.max(12, editorPosition.value.x), maxX),
    y: Math.min(Math.max(12, editorPosition.value.y), maxY),
  };
}

// 移动编辑器到面板下方
function moveEditorNearPanel(force = false): void {
  if (hasEditorBeenDragged.value && !force) {
    return;
  }

  const panel = panelRef.value;

  if (!panel) {
    moveEditorToDefault();

    return;
  }

  const panelRect = panel.getBoundingClientRect();

  const rawWidth = panelRect.width * 0.9;

  const rawHeight = panelRect.height * 0.5;

  const clampedWidth = Math.min(Math.max(260, rawWidth), window.innerWidth - 32);

  const clampedHeight = Math.min(Math.max(220, rawHeight), window.innerHeight - 48);

  const left = panelRect.left + panelRect.width / 2 - clampedWidth / 2;

  const bottomAlignedTop = panelRect.bottom - clampedHeight;

  const safeTop = Math.min(bottomAlignedTop, window.innerHeight - clampedHeight - 16);

  editorSize.value = {
    width: clampedWidth,
    height: clampedHeight,
  };

  editorPosition.value = {
    x: Math.max(16, left),
    y: Math.max(16, safeTop),
  };

  clampEditorWithinViewport();

  clampEditorWithinViewport();
}

// Tab 快捷键聚焦源
function handleTabFocusShortcut(event: KeyboardEvent): void {
  if (event.key !== 'Tab') {
    return;
  }

  if (event.repeat) {
    return;
  }

  event.preventDefault();

  if (sources.value.length === 0) {
    return;
  }

  if (!activeSourceId.value) {
    activeSourceId.value = sources.value[0].id;

    return;
  }

  const currentIndex = sources.value.findIndex(item => item.id === activeSourceId.value);

  const nextIndex = currentIndex >= 0 ? (currentIndex + 1) % sources.value.length : 0;

  activeSourceId.value = sources.value[nextIndex].id;
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

function handleJumpToPage(): void {
  const total = projectPage.value?.pageCount ?? 0;

  if (total <= 0) {
    pageInputValue.value = currentPageIndex.value + 1;

    return;
  }

  const requested = Number(pageInputValue.value);

  if (!Number.isFinite(requested)) {
    pageInputValue.value = currentPageIndex.value + 1;

    return;
  }

  const clamped = Math.min(Math.max(1, Math.round(requested)), total);

  pageInputValue.value = clamped;

  if (clamped - 1 === currentPageIndex.value) {
    return;
  }

  currentPageIndex.value = clamped - 1;

  emit('update:pageIndex', currentPageIndex.value);
}

onMounted(() => {
  window.addEventListener('pointermove', updateDraggingSourcePosition);

  window.addEventListener('pointermove', updateEditorPosition);

  window.addEventListener('pointerup', stopDraggingSource);

  window.addEventListener('pointerup', stopDraggingEditor);

  window.addEventListener('pointercancel', stopDraggingSource);

  window.addEventListener('pointercancel', stopDraggingEditor);

  window.addEventListener('keydown', handleTabFocusShortcut, true);

  window.addEventListener('resize', handleWindowResize);
});

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', updateDraggingSourcePosition);

  window.removeEventListener('pointermove', updateEditorPosition);

  window.removeEventListener('pointerup', stopDraggingSource);

  window.removeEventListener('pointerup', stopDraggingEditor);

  window.removeEventListener('pointercancel', stopDraggingSource);

  window.removeEventListener('pointercancel', stopDraggingEditor);

  window.removeEventListener('keydown', handleTabFocusShortcut, true);

  window.removeEventListener('resize', handleWindowResize);
});
</script>

<template>
  <section class="translator">
    <header class="translator__header">
      <div class="translator__title-group">
        <!-- <span class="translator__badge"
          >第 {{ projectPage?.pageIndex ?? '-' }} / {{ projectPage?.pageCount ?? '-' }} 页</span
        > -->
        <h1 class="translator__title">{{ projectPage?.title ?? '加载中' }}</h1>
      </div>
      <div class="translator__actions">
        <div class="translator__zoom-indicator">{{ Math.round(zoom * 100) }}%</div>
        <form class="translator__page-jump" @submit.prevent="handleJumpToPage">
          <input
            class="page-jump__input"
            type="number"
            min="1"
            :max="projectPage?.pageCount ?? 1"
            v-model.number="pageInputValue"
            :disabled="!projectPage"
            aria-label="跳转页码"
          />
          <span class="page-jump__divider">/ {{ projectPage?.pageCount ?? '-' }}</span>
          <button type="submit" class="page-jump__button" :disabled="!projectPage">跳转</button>
        </form>
      </div>
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
        <button
          type="button"
          class="toolbox__button"
          :title="isPanelCollapsed ? '展开源列表' : '收起源列表'"
          @click="handleTogglePanel"
          :aria-pressed="!isPanelCollapsed"
        >
          <span class="sr-only">{{ isPanelCollapsed ? '展开源列表' : '收起源列表' }}</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              :d="isPanelCollapsed ? 'M15 5L8 12L15 19' : 'M9 5L16 12L9 19'"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </aside>
      <div class="translator__workspace" :style="workspaceStyle">
        <div
          class="board"
          :class="boardGridClass"
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
            <template v-if="showMarkers">
              <button
                v-for="item in sources"
                :key="item.id"
                type="button"
                class="marker"
                :class="{
                  'marker--active': item.id === activeSourceId,
                  'marker--outside': item.category === 'outside',
                  'marker--inside': item.category === 'inside',
                }"
                :data-source-id="item.id"
                :style="{
                  left: `${item.position.x * 100}%`,
                  top: `${item.position.y * 100}%`,
                  transform: markerTransform,
                }"
                @pointerdown="event => handleSourcePointerDown(event, item.id)"
                @click.stop="handleSelectSource(item.id)"
                @contextmenu.prevent.stop="handleRemoveSource(item.id)"
              >
                <span class="marker__label">{{ sourceLabelMap.get(item.id)?.number ?? '' }}</span>
                <span class="marker__pointer" aria-hidden="true"></span>
              </button>
            </template>
          </div>
        </div>
        <aside class="panel" :class="{ 'panel--collapsed': isPanelCollapsed }" ref="panelRef">
          <ul v-if="!isPanelCollapsed" class="panel__list">
            <li
              v-for="item in sources"
              :key="item.id"
              class="panel__item"
              :class="{
                'panel__item--active': item.id === activeSourceId,
                'panel__item--proofed': item.status === 'proofed',
                'panel__item--translated': item.status === 'translated',
              }"
              @click="handleSelectSource(item.id)"
            >
              <div class="panel__item-top">
                <span>{{ sourceLabelMap.get(item.id)?.display ?? '' }}</span>
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
        left: `${editorPosition.x}px`,
        top: `${editorPosition.y}px`,
        width: `${editorSize.width}px`,
        maxHeight: `${editorSize.height}px`,
      }"
    >
      <header class="editor__header" @pointerdown="handleEditorPointerDown">
        <div class="editor__title">
          <span class="editor__label">{{ selectedSourceLabel }}</span>
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
          placeholder="待翻译..."
          ref="translationTextareaRef"
        ></textarea>
        <textarea
          v-model="editorProofText"
          class="editor__textarea editor__textarea--proof"
          placeholder="待校对..."
        ></textarea>
      </div>
      <footer class="editor__footer">
        <div class="editor__footer-actions">
          <button
            type="button"
            class="editor__footer-button editor__footer-button--secondary"
            @click="toggleSourceCategory"
          >
            {{ selectedSource.category === 'inside' ? '切换为框外' : '切换为框内' }}
          </button>
          <button
            type="button"
            class="editor__footer-button editor__footer-button--primary"
            @click="toggleProofStatus"
          >
            {{ selectedSource.status === 'proofed' ? '取消校对状态' : '标记为已校对' }}
          </button>
        </div>
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

.translator__actions {
  display: flex;
  align-items: center;
  gap: 14px;
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

.translator__page-jump {
  display: flex;
  align-items: center;
  gap: 6px;
  border-radius: 999px;
  border: 1px solid rgba(170, 204, 245, 0.65);
  background: rgba(248, 252, 255, 0.92);
  padding: 4px 10px;
}

.page-jump__input {
  width: 56px;
  border: none;
  background: transparent;
  text-align: center;
  font-size: 13px;
  color: #2f3f56;
}

.page-jump__input:focus {
  outline: none;
}

.page-jump__divider {
  font-size: 12px;
  color: #5a6c86;
}

.page-jump__button {
  border: none;
  border-radius: 999px;
  padding: 4px 12px;
  background: rgba(118, 184, 255, 0.85);
  color: #123357;
  font-size: 12px;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease;
}

.page-jump__button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}

.page-jump__button:not(:disabled):hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 12px rgba(118, 184, 255, 0.35);
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
  display: grid;
  grid-template-columns: minmax(0, 1fr) clamp(260px, 22vw, 360px);
  gap: 18px;
  align-items: stretch;
  min-height: 0;
  height: 100%;
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
  cursor: default;
  touch-action: none;
  min-height: 0;
  height: 100%;
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
  cursor: default;
  padding: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  transform-origin: 50% 100%;
  min-width: 0;
  touch-action: none;
}

.marker:focus-visible {
  outline: 2px solid rgba(118, 184, 255, 0.7);
  outline-offset: 4px;
}

.marker__label {
  padding: 6px 14px;
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
  width: 22px;
  height: 18px;
  background: rgba(120, 185, 255, 0.6);
  clip-path: polygon(50% 100%, 0% 0%, 100% 0%);
  filter: drop-shadow(0 6px 10px rgba(94, 138, 196, 0.22));
  pointer-events: none;
}

.marker--inside .marker__label {
  background: rgba(118, 182, 255, 0.95);
  color: #13325c;
  box-shadow: 0 10px 24px rgba(118, 182, 255, 0.36);
}

.marker--inside .marker__pointer {
  background: rgba(118, 182, 255, 0.85);
}

.marker--outside .marker__label {
  background: rgba(255, 185, 210, 0.95);
  color: #61203b;
  box-shadow: 0 10px 24px rgba(255, 185, 210, 0.34);
}

.marker--outside .marker__pointer {
  background: rgba(255, 185, 210, 0.85);
}

.marker--active .marker__label {
  outline: 2px solid rgba(255, 196, 139, 0.6);
  outline-offset: 3px;
}

.panel {
  width: 100%;
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

.panel--collapsed {
  padding: 12px 8px;
  align-items: stretch;
  justify-content: flex-start;
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
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(204, 212, 225, 0.8);
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    border 0.16s ease;
  box-sizing: border-box;
}

.panel__item--active {
  border-color: rgba(118, 184, 255, 0.85);
  box-shadow: 0 12px 26px rgba(150, 189, 246, 0.26);
  transform: translateY(-2px);
}

.panel__item--proofed {
  background: rgba(210, 244, 225, 0.95);
  border-color: rgba(147, 205, 173, 0.7);
}

.panel__item--translated {
  background: rgba(255, 234, 214, 0.95);
  border-color: rgba(250, 191, 143, 0.7);
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
  background: rgba(184, 194, 210, 0.8);
  box-shadow: 0 0 0 3px rgba(184, 194, 210, 0.18);
}

.panel__status-dot--empty {
  background: rgba(192, 200, 215, 0.85);
  box-shadow: 0 0 0 3px rgba(192, 200, 215, 0.2);
}

.panel__status-dot--translated {
  background: rgba(255, 174, 120, 0.85);
  box-shadow: 0 0 0 3px rgba(255, 174, 120, 0.18);
}

.panel__status-dot--proofed {
  background: rgba(122, 204, 159, 0.85);
  box-shadow: 0 0 0 3px rgba(122, 204, 159, 0.2);
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
  width: auto;
  max-width: 520px;
  max-height: 80vh;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.97);
  box-shadow: 0 22px 48px rgba(120, 162, 218, 0.26);
  border: 1px solid rgba(162, 192, 233, 0.4);
  backdrop-filter: blur(10px);
  z-index: 30;
  overflow: hidden;
  display: flex;
  flex-direction: column;
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
  flex: 1;
  overflow: auto;
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
  padding: 12px 14px 14px;
}

.editor__footer-actions {
  display: flex;
  gap: 10px;
}

.editor__footer-button {
  flex: 1;
  border-radius: 14px;
  padding: 10px 0;
  font-size: 13px;
  cursor: pointer;
  border: 1px solid transparent;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    background 0.16s ease;
}

.editor__footer-button--secondary {
  border-color: rgba(170, 204, 245, 0.75);
  background: rgba(236, 244, 255, 0.9);
  color: #325072;
}

.editor__footer-button--secondary:hover {
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 8px 18px rgba(150, 190, 240, 0.25);
}

.editor__footer-button--primary {
  border: none;
  background: linear-gradient(120deg, rgba(124, 205, 182, 0.92), rgba(146, 214, 222, 0.9));
  color: #234060;
}

.editor__footer-button--primary:hover {
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
    display: flex;
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
