<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue';
import AppToast from '../components/AppToast.vue';

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

interface ShortcutHint {
  combo: string;
  description: string;
}

type ToastTone = 'success' | 'error';

interface ToastState {
  message: string;
  tone: ToastTone;
  visible: boolean;
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

const WORKSPACE_BOTTOM_GUTTER = 32;

const MARKER_TIP_OFFSET_PX = 26; // 圆心到指针尖端的偏移，需与样式保持一致

const KEYBOARD_PAN_STEP = 96;

const KEYBOARD_ZOOM_RATIO = 0.25;

const SHORTCUT_HINTS: ShortcutHint[] = [
  {
    combo: 'Tab',
    description: '在标记之间向前轮换焦点',
  },
  {
    combo: 'Shift + Tab',
    description: '在标记之间向后轮换焦点',
  },
  {
    combo: 'Ctrl + Enter',
    description: '在校对模式中提交校对并标记完成',
  },
  {
    combo: 'Esc',
    description: '关闭快捷键悬浮窗',
  },
  {
    combo: 'Ctrl + D',
    description: '切换到下一页',
  },
  {
    combo: 'Ctrl + U',
    description: '切换到上一页',
  },
  {
    combo: 'Ctrl + P',
    description: '在翻译与校对模式之间切换',
  },
  {
    combo: 'Ctrl + F',
    description: '切换自动重定位模式',
  },
  {
    combo: 'Ctrl + L',
    description: '收起或展开源列表',
  },
  {
    combo: 'Ctrl + ↑ / ↓ / ← / →',
    description: '按当前缩放比例平移画布视图',
  },
  {
    combo: 'Ctrl + + / Ctrl + -',
    description: '一次调整 25% 的缩放比例',
  },
  // {
  //   combo: 'Ctrl + Esc',
  //   description: '返回项目详情',
  // },
];

const MOCK_IMAGE_URLS = [
  new URL('../../tests/images/cover_miseyari_01_0001.jpg', import.meta.url).href,
  new URL('../../tests/images/miseyari_01_0001.jpg', import.meta.url).href,
  new URL('../../tests/images/miseyari_02_0001.jpg', import.meta.url).href,
  new URL('../../tests/images/miseyari_03_0001.jpg', import.meta.url).href,
  new URL('../../tests/images/miseyari_04_0001.jpg', import.meta.url).href,
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

const activeMode = ref<'translate' | 'proof'>('translate');

const isProofMode = computed(() => activeMode.value === 'proof');

const translationTextareaRef = ref<HTMLTextAreaElement | null>(null);

const proofTextareaRef = ref<HTMLTextAreaElement | null>(null);

const editorPosition = ref({ ...DEFAULT_EDITOR_POSITION });

const editorSize = ref({ ...DEFAULT_EDITOR_SIZE });

const isDraggingEditor = ref(false);

const editorPointerStart = ref({ x: 0, y: 0 });

const editorPositionStart = ref({ ...DEFAULT_EDITOR_POSITION });

const imageWrapperRef = ref<HTMLDivElement | null>(null);

const panelRef = ref<HTMLElement | null>(null);

const boardRef = ref<HTMLElement | null>(null);

const workspaceRef = ref<HTMLElement | null>(null);

const windowSize = ref({ width: window.innerWidth, height: window.innerHeight });

const hasPanelOverride = ref(false);

const isPanelCollapsed = ref(window.innerWidth < PANEL_COLLAPSE_BREAKPOINT);

const isAutoRelocateEnabled = ref(false);

const draggingSourceId = ref<string | null>(null);

const markerDragOffset = ref({ x: 0, y: 0 });

const isCanvasPointerDown = ref(false);

const isPanningCanvas = ref(false);

const canvasPointerId = ref<number | null>(null);

const canvasPointerStart = ref({ x: 0, y: 0 });

const panStart = ref({ x: 0, y: 0 });

const lastCanvasPointerEvent = ref<PointerEvent | null>(null);

const canvasPointerButton = ref<0 | 2 | null>(null);

const panelHeightPx = ref<number | null>(null);

const workspaceHeightPx = ref<number | null>(null);

let sourceSerial = 0;

let latestPageLoadToken = 0;

const pageSourceStore = new Map<number, TranslationSource[]>();

const hasEditorBeenDragged = ref(false);

let boardResizeObserver: ResizeObserver | null = null;

const isShortcutHelpVisible = ref(false);

const shortcutHelpRef = ref<HTMLDivElement | null>(null);

const toastState = reactive<ToastState>({
  message: '',
  tone: 'success',
  visible: false,
});

let toastTimer: number | null = null;

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
  if (typeof workspaceHeightPx.value === 'number') {
    const clampedHeight = Math.max(MIN_WORKSPACE_HEIGHT, workspaceHeightPx.value);

    return {
      minHeight: `${clampedHeight}px`,
      height: `${clampedHeight}px`,
    };
  }

  const fallbackHeight = Math.max(
    MIN_WORKSPACE_HEIGHT,
    windowSize.value.height - WORKSPACE_VERTICAL_OFFSET
  );

  return {
    minHeight: `${fallbackHeight}px`,
    height: `${fallbackHeight}px`,
  };
});

const boardGridClass = computed(() => ({ 'board--full': isPanelCollapsed.value }));

const panelStyle = computed(() => {
  const style: Record<string, string> = {};

  if (isPanelCollapsed.value) {
    style.flexBasis = `${PANEL_COLLAPSED_WIDTH}px`;
    style.maxWidth = `${PANEL_COLLAPSED_WIDTH}px`;
    style.minWidth = '0px';
  } else {
    const panelPercent = Math.round(PANEL_WIDTH_RATIO * 100);

    style.flexBasis = `clamp(260px, ${panelPercent}%, 520px)`;
    style.maxWidth = '520px';
    style.minWidth = '260px';
  }

  style.height = panelHeightPx.value ? `${panelHeightPx.value}px` : '100%';

  return style;
});

const markerViewportScale = computed(() => {
  const widthFactor = windowSize.value.width / MARKER_REFERENCE_WIDTH;

  return Math.min(MAX_MARKER_VIEWPORT_SCALE, Math.max(MIN_MARKER_VIEWPORT_SCALE, widthFactor));
});

const markerScale = computed(() => markerViewportScale.value / zoom.value);

const markerTransform = computed(() => {
  const scale = markerScale.value;

  const offset = MARKER_TIP_OFFSET_PX * scale;

  return `translate(-50%, calc(-50% - ${offset}px)) scale(${scale})`;
});

const selectedSource = computed(() => {
  if (!activeSourceId.value) {
    return null;
  }

  const target = sources.value.find(item => item.id === activeSourceId.value) ?? null;

  return target;
});

const proofOverlayStyle = computed(() => {
  if (!isProofMode.value || !selectedSource.value) {
    return {} as Record<string, string>;
  }

  const { x, y } = selectedSource.value.position;

  return {
    left: `${x * 100}%`,
    top: `${y * 100}%`,
  } satisfies Record<string, string>;
});

const sourceLabelMap = computed(() => {
  const labelMap = new Map<string, SourceLabelInfo>();

  let index = 1;

  for (const item of sources.value) {
    const number = String(index);

    const suffix = item.category === 'inside' ? '框内' : '框外';

    labelMap.set(item.id, {
      number,
      display: `${number} ${suffix}`,
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

// 切换翻译/校对模式
function handleToggleMode(): void {
  const nextMode: 'translate' | 'proof' = isProofMode.value ? 'translate' : 'proof';

  activeMode.value = nextMode;

  showToast(nextMode === 'proof' ? '已进入校对模式' : '已切换为翻译模式');
}

// 切换自动重定位
function handleToggleAutoRelocate(): void {
  const nextState = !isAutoRelocateEnabled.value;

  isAutoRelocateEnabled.value = nextState;

  showToast(nextState ? '自动重定位已开启' : '自动重定位已关闭');
}

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
    if (isProofMode.value) {
      proofTextareaRef.value?.focus();

      return;
    }

    translationTextareaRef.value?.focus();
  });

  if (source && isAutoRelocateEnabled.value) {
    nextTick(() => {
      centerSourceOnBoard(source);
    });
  }
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

watch(activeMode, mode => {
  if (!selectedSource.value) {
    return;
  }

  nextTick(() => {
    if (mode === 'proof') {
      proofTextareaRef.value?.focus();

      return;
    }

    translationTextareaRef.value?.focus();
  });
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

  updateWorkspaceHeight();

  syncPanelHeightWithBoard();
});

watch(isPanelCollapsed, () => {
  nextTick(() => {
    syncPanelHeightWithBoard();

    updateWorkspaceHeight();
  });
});

watch(isAutoRelocateEnabled, value => {
  if (!value || !selectedSource.value) {
    return;
  }

  nextTick(() => {
    if (!selectedSource.value) {
      return;
    }

    centerSourceOnBoard(selectedSource.value);
  });
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
function addSourceAtPointer(
  event: MouseEvent | PointerEvent,
  category: SourceCategory,
  activate = true
): void {
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

  if (activate) {
    activeSourceId.value = newSource.id;
  }
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

    syncPanelHeightWithBoard();

    updateWorkspaceHeight();
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

  const direction = event.deltaY > 0 ? -1 : 1;

  const candidate = zoom.value + direction * ZOOM_STEP;

  applyZoomWithAnchor(candidate, event.clientX, event.clientY);
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

  const shouldActivate = pointerButton !== 2;

  addSourceAtPointer(referenceEvent, category, shouldActivate);
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

  updateWorkspaceHeight();

  syncPanelHeightWithBoard();
}

// 根据画板高度同步面板高度，保持两侧一致
function syncPanelHeightWithBoard(): void {
  const boardElement = boardRef.value;

  if (!boardElement) {
    panelHeightPx.value = null;

    return;
  }

  const rect = boardElement.getBoundingClientRect();

  panelHeightPx.value = Math.round(rect.height);
}

// 同步工作区高度以减少底部留白
function updateWorkspaceHeight(): void {
  const workspaceElement = workspaceRef.value;

  if (!workspaceElement) {
    workspaceHeightPx.value = null;

    return;
  }

  const rect = workspaceElement.getBoundingClientRect();

  const availableHeight = window.innerHeight - rect.top - WORKSPACE_BOTTOM_GUTTER;

  workspaceHeightPx.value = Math.max(MIN_WORKSPACE_HEIGHT, Math.floor(availableHeight));
}

// 图片加载完同步布局尺寸
function handleBoardContentLoaded(): void {
  syncPanelHeightWithBoard();

  updateWorkspaceHeight();
}

// 根据当前源进行画面居中
function centerSourceOnBoard(source: TranslationSource): void {
  const boardElement = boardRef.value;

  const wrapper = imageWrapperRef.value;

  if (!boardElement || !wrapper) {
    return;
  }

  const baseWidth = wrapper.offsetWidth;

  const baseHeight = wrapper.offsetHeight;

  if (baseWidth === 0 || baseHeight === 0) {
    return;
  }

  const markerX = source.position.x * baseWidth;

  const markerY = source.position.y * baseHeight;

  const nextPanX = baseWidth / 2 - markerX * zoom.value;

  const nextPanY = baseHeight / 2 - markerY * zoom.value;

  pan.value = {
    x: nextPanX,
    y: nextPanY,
  };
}

function showToast(message: string, tone: ToastTone = 'success', duration = 2000): void {
  toastState.message = message;
  toastState.tone = tone;
  toastState.visible = true;

  if (toastTimer !== null) {
    window.clearTimeout(toastTimer);
  }

  toastTimer = window.setTimeout(() => {
    toastState.visible = false;
    toastTimer = null;
  }, duration);
}

// 根据锚点调整缩放
function applyZoomWithAnchor(targetZoom: number, anchorX: number, anchorY: number): void {
  const wrapper = imageWrapperRef.value;

  if (!wrapper) {
    return;
  }

  const clamped = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Number(targetZoom.toFixed(4))));

  if (clamped === zoom.value) {
    return;
  }

  const rect = wrapper.getBoundingClientRect();

  const baseLeft = rect.left - pan.value.x;

  const baseTop = rect.top - pan.value.y;

  const offsetX = anchorX - rect.left;

  const offsetY = anchorY - rect.top;

  const baseX = offsetX / zoom.value;

  const baseY = offsetY / zoom.value;

  const nextPanX = anchorX - baseLeft - clamped * baseX;

  const nextPanY = anchorY - baseTop - clamped * baseY;

  pan.value = {
    x: nextPanX,
    y: nextPanY,
  };

  zoom.value = clamped;
}

// 处理键盘平移
function handleKeyboardPan(deltaX: number, deltaY: number): void {
  pan.value = {
    x: pan.value.x + deltaX,
    y: pan.value.y + deltaY,
  };
}

// 处理键盘缩放
function handleKeyboardZoom(direction: 1 | -1): void {
  if (!projectPage.value) {
    return;
  }

  const boardElement = boardRef.value;

  if (!boardElement) {
    return;
  }

  const boardRect = boardElement.getBoundingClientRect();

  const anchorX = boardRect.left + boardRect.width / 2;

  const anchorY = boardRect.top + boardRect.height / 2;

  const factor = direction > 0 ? 1 + KEYBOARD_ZOOM_RATIO : 1 - KEYBOARD_ZOOM_RATIO;

  const nextZoom = zoom.value * factor;

  applyZoomWithAnchor(nextZoom, anchorX, anchorY);
}

// 选中标记
function handleSelectSource(sourceId: string): void {
  if (activeSourceId.value === sourceId) {
    if (!isAutoRelocateEnabled.value) {
      return;
    }

    const target = sources.value.find(item => item.id === sourceId);

    if (!target) {
      return;
    }

    nextTick(() => {
      centerSourceOnBoard(target);
    });

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

// 打开快捷键悬浮窗
function openShortcutHelp(): void {
  isShortcutHelpVisible.value = true;

  nextTick(() => {
    shortcutHelpRef.value?.focus();
  });
}

// 关闭快捷键悬浮窗
function closeShortcutHelp(): void {
  isShortcutHelpVisible.value = false;
}

// 处理快捷键悬浮窗键盘事件
function handleShortcutHelpKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') {
    return;
  }

  event.preventDefault();

  closeShortcutHelp();
}

// 处理全局快捷键
function handleGlobalShortcuts(event: KeyboardEvent): void {
  if (event.repeat) {
    return;
  }

  if (event.key === 'Tab') {
    if (sources.value.length === 0) {
      return;
    }

    event.preventDefault();

    if (!activeSourceId.value) {
      activeSourceId.value = event.shiftKey
        ? sources.value[sources.value.length - 1].id
        : sources.value[0].id;

      return;
    }

    const currentIndex = sources.value.findIndex(item => item.id === activeSourceId.value);

    if (currentIndex < 0) {
      activeSourceId.value = event.shiftKey
        ? sources.value[sources.value.length - 1].id
        : sources.value[0].id;

      return;
    }

    const direction = event.shiftKey ? -1 : 1;

    const nextIndex = (currentIndex + direction + sources.value.length) % sources.value.length;

    activeSourceId.value = sources.value[nextIndex].id;

    return;
  }

  if (event.ctrlKey && !event.metaKey && !event.altKey) {
    const key = event.key.toLowerCase();

    if (key === 'arrowup' || key === 'arrowdown' || key === 'arrowleft' || key === 'arrowright') {
      event.preventDefault();

      const step = KEYBOARD_PAN_STEP / zoom.value;

      if (key === 'arrowup') {
        handleKeyboardPan(0, step);

        return;
      }

      if (key === 'arrowdown') {
        handleKeyboardPan(0, -step);

        return;
      }

      if (key === 'arrowleft') {
        handleKeyboardPan(step, 0);

        return;
      }

      handleKeyboardPan(-step, 0);

      return;
    }

    if (event.key === '+' || event.key === '=' || key === 'add') {
      event.preventDefault();

      handleKeyboardZoom(1);

      return;
    }

    if (event.key === '-' || event.key === '_' || key === 'subtract') {
      event.preventDefault();

      handleKeyboardZoom(-1);

      return;
    }

    if (key === 'p') {
      event.preventDefault();

      handleToggleMode();

      return;
    }

    if (key === 'f') {
      event.preventDefault();

      handleToggleAutoRelocate();

      return;
    }

    if (key === 'l') {
      event.preventDefault();

      handleTogglePanel();

      return;
    }

    if (key === 'd') {
      event.preventDefault();

      handleNextPage();

      return;
    }

    if (key === 'u') {
      event.preventDefault();

      handlePreviousPage();

      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();

      handleBackToProject();

      return;
    }
  }
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

function handleProofShortcut(event: KeyboardEvent): void {
  if (!isProofMode.value) {
    return;
  }

  if (event.key !== 'Enter' || !event.ctrlKey || event.shiftKey || event.altKey || event.metaKey) {
    return;
  }

  event.preventDefault();

  toggleProofStatus();
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

  window.addEventListener('keydown', handleGlobalShortcuts, true);

  window.addEventListener('resize', handleWindowResize);

  nextTick(syncPanelHeightWithBoard);

  nextTick(updateWorkspaceHeight);

  if (typeof ResizeObserver !== 'undefined') {
    boardResizeObserver = new ResizeObserver(() => {
      syncPanelHeightWithBoard();

      updateWorkspaceHeight();
    });

    nextTick(() => {
      if (boardRef.value) {
        boardResizeObserver?.observe(boardRef.value);
      }
    });
  }
});

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', updateDraggingSourcePosition);

  window.removeEventListener('pointermove', updateEditorPosition);

  window.removeEventListener('pointerup', stopDraggingSource);

  window.removeEventListener('pointerup', stopDraggingEditor);

  window.removeEventListener('pointercancel', stopDraggingSource);

  window.removeEventListener('pointercancel', stopDraggingEditor);

  window.removeEventListener('keydown', handleGlobalShortcuts, true);

  window.removeEventListener('resize', handleWindowResize);

  if (boardResizeObserver) {
    boardResizeObserver.disconnect();

    boardResizeObserver = null;
  }

  if (toastTimer !== null) {
    window.clearTimeout(toastTimer);

    toastTimer = null;
  }
});
</script>

<template>
  <section class="translator">
    <header class="translator__header">
      <div class="translator__title-group">
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
              :d="isPanelCollapsed ? 'M7 8H17M7 12H17M7 16H17' : 'M8 9H16M8 12H16M8 15H16'"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="toolbox__button"
          :class="{ 'toolbox__button--active': isAutoRelocateEnabled }"
          :title="isAutoRelocateEnabled ? '关闭自动重定位' : '开启自动重定位'"
          @click="handleToggleAutoRelocate"
          :aria-pressed="isAutoRelocateEnabled"
        >
          <span class="sr-only">{{
            isAutoRelocateEnabled ? '关闭自动重定位' : '开启自动重定位'
          }}</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="5.2" fill="none" stroke="currentColor" stroke-width="1.8" />
            <path
              d="M12 4V6.4M12 17.6V20M6.4 12H4M20 12H17.6"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="toolbox__button"
          :class="{ 'toolbox__button--active': isProofMode }"
          :title="isProofMode ? '切换为翻译模式' : '切换为校对模式'"
          @click="handleToggleMode"
          :aria-pressed="isProofMode"
        >
          <span class="sr-only">{{ isProofMode ? '切换为翻译模式' : '切换为校对模式' }}</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M6 12.5L10 16.5L18 8.5M11 4H7A3 3 0 0 0 4 7V17A3 3 0 0 0 7 20H17A3 3 0 0 0 20 17V13"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button type="button" class="toolbox__button" title="快捷键指南" @click="openShortcutHelp">
          <span class="sr-only">快捷键指南</span>
          <svg class="toolbox__icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M12 3C7.58 3 4 6.58 4 11C4 15.42 7.58 19 12 19C16.42 19 20 15.42 20 11C20 6.58 16.42 3 12 3Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M10.8 9.2C10.8 8.32 11.52 7.6 12.4 7.6C13.28 7.6 14 8.32 14 9.2C14 10.08 13.48 10.48 13.08 10.76C12.48 11.18 12.2 11.48 12.2 12.2V12.6"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M12 15.2H12.01"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </aside>
      <div class="translator__workspace" :style="workspaceStyle" ref="workspaceRef">
        <div
          class="board"
          :class="[boardGridClass, { 'board--proof': isProofMode }]"
          ref="boardRef"
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
              @load="handleBoardContentLoaded"
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
                :aria-label="sourceLabelMap.get(item.id)?.display ?? ''"
                @pointerdown="event => handleSourcePointerDown(event, item.id)"
                @click.stop="handleSelectSource(item.id)"
                @contextmenu.prevent.stop="handleRemoveSource(item.id)"
              >
                <span class="marker__label">{{ sourceLabelMap.get(item.id)?.number ?? '' }}</span>
                <span class="marker__pointer" aria-hidden="true"></span>
              </button>
            </template>
            <div
              v-if="isProofMode && selectedSource"
              class="marker-overlay"
              :style="proofOverlayStyle"
            >
              <div class="marker-overlay__content">
                {{ selectedSource.translationText || '〈empty〉' }}
              </div>
            </div>
          </div>
        </div>
        <aside
          class="panel"
          :class="{ 'panel--collapsed': isPanelCollapsed }"
          ref="panelRef"
          :style="panelStyle"
        >
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
                  item.status === 'proofed'
                    ? item.proofText || '〈empty〉'
                    : item.translationText || '〈empty〉'
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
          <span class="editor__mode">{{ isProofMode ? '校对模式' : '翻译模式' }}</span>
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
      <div class="editor__body" :class="{ 'editor__body--proof-mode': isProofMode }">
        <div v-if="!isProofMode" class="editor__field">
          <div class="editor__field-label">翻译稿</div>
          <textarea
            v-model="editorTranslationText"
            class="editor__textarea"
            placeholder="待翻译..."
            ref="translationTextareaRef"
          ></textarea>
        </div>
        <div v-if="isProofMode" class="editor__field editor__field--proof">
          <div class="editor__field-label">校对稿</div>
          <textarea
            v-model="editorProofText"
            class="editor__textarea editor__textarea--proof"
            placeholder="待校对..."
            ref="proofTextareaRef"
            @keydown="handleProofShortcut"
          ></textarea>
        </div>
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
            {{
              selectedSource.status === 'proofed'
                ? '取消校对状态'
                : isProofMode
                  ? '提交校对'
                  : '标记为已校对'
            }}
          </button>
        </div>
      </footer>
    </div>
    <div
      v-if="isShortcutHelpVisible"
      class="shortcut-help-overlay"
      @click="closeShortcutHelp"
    ></div>
    <div
      v-if="isShortcutHelpVisible"
      class="shortcut-help"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-label="快捷键指南"
      ref="shortcutHelpRef"
      @keydown="handleShortcutHelpKeydown"
      @click.stop
    >
      <header class="shortcut-help__header">
        <h2 class="shortcut-help__title">快捷键指南</h2>
        <button type="button" class="shortcut-help__close" @click="closeShortcutHelp">
          <span class="sr-only">关闭快捷键悬浮窗</span>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M8 8L16 16M16 8L8 16" stroke-width="1.8" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <ul class="shortcut-help__list">
        <li v-for="item in SHORTCUT_HINTS" :key="item.combo" class="shortcut-help__item">
          <span class="shortcut-help__combo">{{ item.combo }}</span>
          <span class="shortcut-help__description">{{ item.description }}</span>
        </li>
      </ul>
    </div>
    <AppToast :visible="toastState.visible" :message="toastState.message" :tone="toastState.tone" />
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
  background: rgba(200, 215, 230, 0.85);
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

.toolbox__button--active {
  border-color: rgba(118, 184, 255, 0.85);
  box-shadow: 0 12px 22px rgba(118, 184, 255, 0.26);
  background: rgba(228, 245, 255, 0.95);
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
  height: 100%;
}

.translator__workspace > * {
  height: 100%;
  min-height: 0;
}

.board {
  position: relative;
  flex: 1 1 0;
  min-width: 0;
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

.board--proof {
  border: 1px solid rgba(255, 182, 193, 0.85);
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
  background: rgba(255, 255, 255, 0.95);
  cursor: pointer;
  padding: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  /* increased by ~70% from 24px -> 41px */
  width: 28px;
  height: 28px;
  border-radius: 50%;
  transform-origin: 50% 50%;
  box-shadow: 0 10px 22px rgba(20, 24, 32, 0.14);
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    background 0.16s ease,
    border 0.16s ease;
  /* narrow dark stroke */
  border: 1px solid rgba(202, 205, 212, 0.92);
  min-width: 0;
  touch-action: none;
  overflow: visible;
}

.marker:hover,
.marker:focus-visible {
  box-shadow: 0 14px 30px rgba(118, 166, 219, 0.32);
  outline: none;
}

.marker__label {
  color: #173556;
  /* scale label a bit to match larger marker */
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.2px;
  pointer-events: none;
}

.marker__pointer {
  position: absolute;
  /* moved downward to sit visually attached to larger circle */
  bottom: -12px;
  left: 50%;
  transform: translateX(-50%);
  /* scaled by ~70% from 14x8 -> 24x14 */
  width: 16px;
  height: 14px;
  background: rgba(202, 208, 214, 0.8);
  clip-path: polygon(50% 100%, 0 0, 100% 0);
  filter: drop-shadow(0 3px 6px rgba(20, 24, 32, 0.18));
  pointer-events: none;
}

.marker--inside {
  /* pink-ish fill for inside markers */
  background: linear-gradient(145deg, rgba(255, 182, 193, 0.96), rgba(255, 150, 185, 0.9));
  border-color: rgba(160, 40, 80, 0.92);
}

.marker--inside .marker__label {
  color: #13325c;
}

.marker--inside .marker__pointer {
  background: rgba(255, 160, 185, 0.95);
}

.marker--outside {
  /* pale yellow for outside markers */
  background: linear-gradient(145deg, rgba(255, 249, 196, 0.96), rgba(255, 241, 160, 0.9));
  border-color: rgba(180, 150, 60, 0.92);
}

.marker--outside .marker__label {
  color: #61203b;
}

.marker--outside .marker__pointer {
  background: rgba(255, 235, 150, 0.95);
}

.marker--active {
  box-shadow: 0 16px 34px rgba(255, 196, 139, 0.4);
  border-color: rgba(255, 196, 139, 0.9);
}

.marker--active .marker__pointer {
  background: rgba(255, 196, 139, 0.95);
}

.marker-overlay {
  position: absolute;
  transform: translate(-50%, calc(-100% - 14px));
  pointer-events: none;
  max-width: 240px;
  width: max-content;
}

.marker-overlay__content {
  padding: 8px 12px;
  border-radius: 12px;
  background: rgba(28, 40, 58, 0.72);
  color: #f5f8ff;
  font-size: 12px;
  line-height: 1.5;
  backdrop-filter: blur(6px);
  box-shadow: 0 10px 24px rgba(30, 50, 76, 0.32);
  white-space: pre-line;
}

.panel {
  flex: 0 0 auto;
  padding: 18px 18px 16px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.88);
  box-shadow: 0 24px 50px rgba(123, 168, 225, 0.18);
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  box-sizing: border-box;
  transition:
    flex-basis 0.28s ease,
    max-width 0.28s ease,
    padding 0.28s ease;
}

.panel--collapsed {
  padding: 12px 8px;
  align-items: stretch;
  justify-content: flex-start;
  box-sizing: border-box;
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

.editor__mode {
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  letter-spacing: 0.3px;
  background: rgba(133, 182, 255, 0.18);
  color: #2a4f7a;
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

.editor__body--proof-mode {
  gap: 16px;
}

.editor__field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.editor__field-label {
  font-size: 12px;
  color: #4a5f7d;
  font-weight: 600;
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

.shortcut-help-overlay {
  position: fixed;
  inset: 0;
  background: rgba(30, 46, 70, 0.28);
  backdrop-filter: blur(2px);
  z-index: 45;
}

.shortcut-help {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: min(440px, calc(100vw - 48px));
  max-height: min(420px, calc(100vh - 96px));
  padding: 22px 26px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.97);
  box-shadow: 0 28px 56px rgba(104, 146, 204, 0.32);
  border: 1px solid rgba(160, 192, 236, 0.5);
  display: flex;
  flex-direction: column;
  gap: 18px;
  outline: none;
  z-index: 50;
}

.shortcut-help__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.shortcut-help__title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #244163;
}

.shortcut-help__close {
  border: none;
  background: transparent;
  padding: 6px;
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.16s ease;
  color: #3b5a80;
}

.shortcut-help__close:hover {
  background: rgba(229, 239, 255, 0.8);
}

.shortcut-help__close svg {
  width: 18px;
  height: 18px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
}

.shortcut-help__list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
}

.shortcut-help__item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 14px;
  border-radius: 14px;
  background: rgba(242, 248, 255, 0.92);
  border: 1px solid rgba(186, 210, 244, 0.6);
}

.shortcut-help__combo {
  flex: none;
  min-width: 120px;
  padding: 6px 10px;
  border-radius: 10px;
  background: rgba(134, 188, 255, 0.22);
  color: #24507a;
  font-weight: 600;
  font-size: 13px;
  text-align: center;
}

.shortcut-help__description {
  flex: 1;
  font-size: 13px;
  color: #30455f;
  line-height: 1.5;
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
