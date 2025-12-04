<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { uploadProjectFile } from '../ipc/project';
import { useUploadStore } from '../stores/upload';
import { useToastStore } from '../stores/toast';

const props = defineProps<{
  projectId: string;
  projectName: string;
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  uploadComplete: [successCount: number, failedCount: number];
}>();

const uploadStore = useUploadStore();
const toastStore = useToastStore();

const fileInput = ref<HTMLInputElement | null>(null);
const isUploading = ref(false);

// 获取当前项目的上传组
const uploadGroup = computed(() => uploadStore.uploadGroups.get(props.projectId));

// 监听visible变化，关闭时清理状态
watch(
  () => props.visible,
  newVal => {
    if (!newVal && uploadGroup.value) {
      // 如果所有任务都已完成（成功或失败），清理记录
      const allDone = uploadGroup.value.tasks.every(
        t => t.status === 'success' || t.status === 'failed'
      );
      if (allDone && uploadStore.activeUploads === 0) {
        uploadStore.clearProject(props.projectId);
      }
    }
  }
);

// 触发文件选择
function handleSelectFiles(): void {
  fileInput.value?.click();
}

// 处理文件选择
async function handleFilesSelected(event: Event): Promise<void> {
  const target = event.target as HTMLInputElement;
  const files = target.files;
  if (!files || files.length === 0) return;

  // 验证文件类型
  const allowedExts = ['jpg', 'jpeg', 'png', 'bmp'];
  const validFiles: Array<{ fileName: string; filePath: string; file: File }> = [];

  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    const ext = file.name.split('.').pop()?.toLowerCase() || '';
    if (allowedExts.includes(ext)) {
      validFiles.push({
        fileName: file.name,
        filePath: file.name, // 前端使用文件名作为标识
        file: file,
      });
    } else {
      toastStore.show(`文件 ${file.name} 格式不支持，仅支持 jpg/jpeg/png/bmp`, 'error');
    }
  }

  if (validFiles.length === 0) {
    return;
  }

  // 添加任务到队列
  uploadStore.addTasks(
    props.projectId,
    props.projectName,
    validFiles.map(f => ({ fileName: f.fileName, filePath: f.filePath }))
  );

  // 启动上传
  isUploading.value = true;
  await processUploadQueue(validFiles);
  isUploading.value = false;

  // 清空文件输入
  target.value = '';

  // 通知完成
  if (uploadGroup.value) {
    emit('uploadComplete', uploadGroup.value.successCount, uploadGroup.value.failedCount);

    if (uploadGroup.value.failedCount > 0) {
      toastStore.show(
        `上传完成：成功 ${uploadGroup.value.successCount}，失败 ${uploadGroup.value.failedCount}`,
        'error'
      );
    } else {
      toastStore.show(`上传完成：全部 ${uploadGroup.value.successCount} 个文件成功`);
    }
  }
}

// 处理上传队列（5并发）
async function processUploadQueue(
  files: Array<{ fileName: string; filePath: string; file: File }>
): Promise<void> {
  const CONCURRENCY = 5;
  const tasks = uploadGroup.value?.tasks || [];

  const uploaders: Array<Promise<void>> = [];

  for (let i = 0; i < files.length; i++) {
    const fileData = files[i];
    const task = tasks.find(t => t.fileName === fileData.fileName && t.status === 'pending');
    if (!task) continue;

    // 等待并发槽位
    if (uploaders.length >= CONCURRENCY) {
      await Promise.race(uploaders);
      uploaders.splice(
        uploaders.findIndex(p => p === undefined),
        1
      );
    }

    const uploadPromise = uploadSingleFile(task.id, fileData.file).finally(() => {
      const idx = uploaders.indexOf(uploadPromise);
      if (idx !== -1) uploaders.splice(idx, 1);
    });

    uploaders.push(uploadPromise);
  }

  // 等待所有上传完成
  await Promise.all(uploaders);
}

// 上传单个文件
async function uploadSingleFile(taskId: string, file: File): Promise<void> {
  uploadStore.updateTask(taskId, { status: 'uploading', progress: 0 });
  uploadStore.incrementActive();

  try {
    // 读取文件字节
    const arrayBuffer = await file.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);

    // 模拟进度（实际上传无法获取真实进度）
    uploadStore.updateTask(taskId, { progress: 50 });

    // 调用 IPC 上传
    await uploadProjectFile(props.projectId, file.name, bytes);

    // 上传成功
    uploadStore.updateTask(taskId, { status: 'success', progress: 100 });
  } catch (err: any) {
    console.error('Upload failed:', err);
    uploadStore.updateTask(taskId, {
      status: 'failed',
      progress: 0,
      error: err?.message || String(err),
    });
  } finally {
    uploadStore.decrementActive();
  }
}

// 重试失败的文件
async function handleRetryFailed(): Promise<void> {
  const failedTasks = uploadStore.getFailedTasks(props.projectId);
  if (failedTasks.length === 0) return;

  isUploading.value = true;

  // 重置失败任务状态
  for (const task of failedTasks) {
    uploadStore.updateTask(task.id, { status: 'pending', error: undefined, progress: 0 });
  }

  // 需要重新读取文件，但由于前端无法保留 File 对象，这里提示用户重新选择
  toastStore.show('请重新选择失败的文件进行上传');
  isUploading.value = false;
}

// 清除完成的任务
function handleClearCompleted(): void {
  if (!uploadGroup.value) return;

  const group = uploadGroup.value;
  group.tasks = group.tasks.filter(t => t.status === 'uploading' || t.status === 'pending');
  group.totalCount = group.tasks.length;
  group.successCount = 0;
  group.failedCount = 0;

  if (group.tasks.length === 0) {
    uploadStore.clearProject(props.projectId);
  }
}

// 关闭对话框
function handleClose(): void {
  if (isUploading.value) {
    toastStore.show('上传进行中，无法关闭', 'error');
    return;
  }
  emit('close');
}
</script>

<template>
  <div v-if="visible" class="uploader-overlay" @click.self="handleClose">
    <div class="uploader-dialog">
      <div class="dialog-header">
        <h3>上传漫画页</h3>
        <button class="close-btn" @click="handleClose">✕</button>
      </div>

      <div class="dialog-body">
        <div class="project-info">
          <span class="label">项目：</span>
          <span class="value">{{ projectName }}</span>
        </div>

        <div class="upload-section">
          <button class="select-btn" @click="handleSelectFiles" :disabled="isUploading">
            选择文件
          </button>
          <span class="hint">支持 jpg/jpeg/png/bmp 格式，最多5个文件同时上传</span>
        </div>

        <input
          ref="fileInput"
          type="file"
          accept=".jpg,.jpeg,.png,.bmp"
          multiple
          style="display: none"
          @change="handleFilesSelected"
        />

        <!-- 上传任务列表 -->
        <div v-if="uploadGroup && uploadGroup.tasks.length > 0" class="task-list">
          <div class="task-header">
            <span>文件列表 ({{ uploadGroup.totalCount }})</span>
            <div class="task-actions">
              <button
                v-if="uploadGroup.failedCount > 0"
                class="retry-btn"
                @click="handleRetryFailed"
                :disabled="isUploading"
              >
                重试失败 ({{ uploadGroup.failedCount }})
              </button>
              <button
                v-if="uploadGroup.successCount > 0"
                class="clear-btn"
                @click="handleClearCompleted"
                :disabled="isUploading"
              >
                清除完成
              </button>
            </div>
          </div>

          <div class="task-items">
            <div v-for="task in uploadGroup.tasks" :key="task.id" class="task-item">
              <div class="task-info">
                <span class="task-name">{{ task.fileName }}</span>
                <span class="task-status" :class="`status-${task.status}`">
                  {{ task.status === 'pending' ? '等待' : '' }}
                  {{ task.status === 'uploading' ? '上传中' : '' }}
                  {{ task.status === 'success' ? '✓' : '' }}
                  {{ task.status === 'failed' ? '失败' : '' }}
                </span>
              </div>
              <div v-if="task.status === 'uploading'" class="task-progress">
                <div class="progress-bar" :style="{ width: `${task.progress}%` }"></div>
              </div>
              <div v-if="task.error" class="task-error">{{ task.error }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.uploader-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.uploader-dialog {
  background: white;
  border-radius: 8px;
  width: 600px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e0e0e0;
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.close-btn:hover {
  background: #f0f0f0;
}

.dialog-body {
  padding: 20px;
  overflow-y: auto;
}

.project-info {
  margin-bottom: 16px;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 4px;
}

.project-info .label {
  font-weight: 600;
  color: #666;
}

.project-info .value {
  color: #333;
}

.upload-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.select-btn {
  padding: 10px 20px;
  background: #2f6fb2;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.select-btn:hover:not(:disabled) {
  background: #255a8f;
}

.select-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.hint {
  font-size: 12px;
  color: #999;
}

.task-list {
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f9f9f9;
  border-bottom: 1px solid #e0e0e0;
  font-weight: 600;
}

.task-actions {
  display: flex;
  gap: 8px;
}

.retry-btn,
.clear-btn {
  padding: 6px 12px;
  font-size: 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.retry-btn {
  background: #ff9800;
  color: white;
}

.retry-btn:hover:not(:disabled) {
  background: #f57c00;
}

.clear-btn {
  background: #e0e0e0;
  color: #333;
}

.clear-btn:hover:not(:disabled) {
  background: #d0d0d0;
}

.retry-btn:disabled,
.clear-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.task-items {
  max-height: 300px;
  overflow-y: auto;
}

.task-item {
  padding: 12px 16px;
  border-bottom: 1px solid #e0e0e0;
}

.task-item:last-child {
  border-bottom: none;
}

.task-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.task-name {
  font-size: 14px;
  color: #333;
  font-weight: 500;
}

.task-status {
  font-size: 12px;
  font-weight: 600;
}

.status-pending {
  color: #999;
}

.status-uploading {
  color: #2196f3;
}

.status-success {
  color: #4caf50;
}

.status-failed {
  color: #f44336;
}

.task-progress {
  height: 4px;
  background: #e0e0e0;
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 4px;
}

.progress-bar {
  height: 100%;
  background: #2196f3;
  transition: width 0.3s ease;
}

.task-error {
  font-size: 12px;
  color: #f44336;
  margin-top: 4px;
}
</style>
