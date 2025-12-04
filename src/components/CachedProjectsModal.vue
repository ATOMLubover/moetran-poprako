<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useToastStore } from '../stores/toast';
import { deleteFileCache, getAllCachedProjects } from '../ipc/image_cache';
import type { CachedProjectMetadata } from '../ipc/image_cache';

const toastStore = useToastStore();

const cachedProjects = ref<CachedProjectMetadata[]>([]);
const isLoading = ref(true);
const deletingIds = ref<Set<string>>(new Set());

const emit = defineEmits<{
  close: [];
}>();

// 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

// 格式化时间戳
function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

// 加载缓存项目列表
async function loadCachedProjects() {
  isLoading.value = true;
  try {
    cachedProjects.value = await getAllCachedProjects();
  } catch (err) {
    console.error('加载缓存项目列表失败', err);
    toastStore.show('加载缓存列表失败', 'error');
  } finally {
    isLoading.value = false;
  }
}

// 删除缓存
async function handleDelete(projectId: string, projectName: string) {
  if (!confirm(`确定删除项目 "${projectName}" 的缓存？`)) {
    return;
  }

  deletingIds.value.add(projectId);
  try {
    await deleteFileCache(projectId);
    toastStore.show('缓存已删除');

    // 重新加载列表
    await loadCachedProjects();
  } catch (err) {
    console.error('删除缓存失败', err);
    toastStore.show('删除缓存失败', 'error');
  } finally {
    deletingIds.value.delete(projectId);
  }
}

onMounted(() => {
  void loadCachedProjects();
});
</script>

<template>
  <div class="cached-modal-overlay" @click="emit('close')">
    <div class="cached-modal-content" @click.stop>
      <div class="cached-modal-header">
        <h2>缓存项目</h2>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div v-if="isLoading" class="loading-container">
        <p>加载中...</p>
      </div>

      <div v-else-if="cachedProjects.length === 0" class="empty-container">
        <p>暂无缓存项目</p>
      </div>

      <div v-else class="cached-list">
        <div
          v-for="project in cachedProjects"
          :key="project.projectId"
          class="cached-item"
          :class="{ deleting: deletingIds.has(project.projectId) }"
        >
          <div class="cached-item-info">
            <div class="cached-item-name">{{ project.projectName }}</div>
            <div class="cached-item-meta">
              <span class="cached-item-time">{{ formatDate(project.cachedAt) }}</span>
              <span class="cached-item-size">{{ formatSize(project.totalSizeBytes) }}</span>
              <span
                class="cached-item-status"
                :class="project.status === 'completed' ? 'status-success' : 'status-failed'"
              >
                {{ project.status === 'completed' ? '完成' : '失败' }}
              </span>
              <span class="cached-item-count">{{ project.fileCount }} 张图片</span>
            </div>
          </div>
          <button
            class="delete-btn"
            :disabled="deletingIds.has(project.projectId)"
            @click="handleDelete(project.projectId, project.projectName)"
          >
            {{ deletingIds.has(project.projectId) ? '删除中...' : '删除' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cached-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.cached-modal-content {
  width: 90%;
  max-width: 700px;
  max-height: 80vh;
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.cached-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid #e0e0e0;
}

.cached-modal-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #333;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: #666;
  font-size: 24px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: #f5f5f5;
}

.loading-container,
.empty-container {
  padding: 60px 24px;
  text-align: center;
  color: #999;
  font-size: 15px;
}

.cached-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

.cached-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: #f9f9f9;
  border-radius: 8px;
  margin-bottom: 12px;
  transition: background 0.2s;
}

.cached-item:hover {
  background: #f0f0f0;
}

.cached-item.deleting {
  opacity: 0.5;
  pointer-events: none;
}

.cached-item-info {
  flex: 1;
  min-width: 0;
}

.cached-item-name {
  font-size: 16px;
  font-weight: 500;
  color: #333;
  margin-bottom: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cached-item-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  color: #666;
}

.cached-item-time {
  color: #999;
}

.cached-item-size {
  font-weight: 500;
}

.cached-item-status {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.status-success {
  background: #e8f5e9;
  color: #2e7d32;
}

.status-failed {
  background: #ffebee;
  color: #c62828;
}

.cached-item-count {
  color: #999;
}

.delete-btn {
  margin-left: 16px;
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.3px;
  border-radius: 999px;
  cursor: pointer;
  border: 1px solid rgba(210, 120, 120, 0.7);
  color: #823c3c;
  background: #fff3f3;
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease;
  white-space: nowrap;
}

.delete-btn:hover:not(:disabled) {
  box-shadow: 0 8px 24px rgba(210, 120, 120, 0.28);
  transform: translateY(-2px);
}

.delete-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}
</style>
