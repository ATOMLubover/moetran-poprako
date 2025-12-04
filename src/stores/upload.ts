import { defineStore } from 'pinia';
import { ref } from 'vue';

// 单个上传任务状态
export interface UploadTask {
  id: string; // 任务唯一标识（通常为文件路径或随机生成）
  projectId: string;
  projectName: string;
  fileName: string;
  filePath: string;
  status: 'pending' | 'uploading' | 'success' | 'failed';
  progress: number; // 0-100
  error?: string;
}

// 上传任务组（按项目分组）
export interface ProjectUploadGroup {
  projectId: string;
  projectName: string;
  tasks: UploadTask[];
  totalCount: number;
  successCount: number;
  failedCount: number;
}

export const useUploadStore = defineStore('upload', () => {
  // 所有上传任务（按项目ID分组）
  const uploadGroups = ref<Map<string, ProjectUploadGroup>>(new Map());

  // 当前活跃的上传任务数
  const activeUploads = ref<number>(0);

  // 添加上传任务
  function addTasks(
    projectId: string,
    projectName: string,
    files: Array<{ fileName: string; filePath: string }>
  ): void {
    if (!uploadGroups.value.has(projectId)) {
      uploadGroups.value.set(projectId, {
        projectId,
        projectName,
        tasks: [],
        totalCount: 0,
        successCount: 0,
        failedCount: 0,
      });
    }

    const group = uploadGroups.value.get(projectId)!;

    for (const file of files) {
      const task: UploadTask = {
        id: `${projectId}-${file.fileName}-${Date.now()}-${Math.random()}`,
        projectId,
        projectName,
        fileName: file.fileName,
        filePath: file.filePath,
        status: 'pending',
        progress: 0,
      };
      group.tasks.push(task);
    }

    group.totalCount = group.tasks.length;
  }

  // 更新任务状态
  function updateTask(taskId: string, updates: Partial<UploadTask>): void {
    for (const group of uploadGroups.value.values()) {
      const task = group.tasks.find(t => t.id === taskId);
      if (task) {
        Object.assign(task, updates);

        // 更新统计
        group.successCount = group.tasks.filter(t => t.status === 'success').length;
        group.failedCount = group.tasks.filter(t => t.status === 'failed').length;
        break;
      }
    }
  }

  // 获取指定项目的失败任务
  function getFailedTasks(projectId: string): UploadTask[] {
    const group = uploadGroups.value.get(projectId);
    return group ? group.tasks.filter(t => t.status === 'failed') : [];
  }

  // 获取所有失败的项目（用于通知）
  function getProjectsWithFailures(): ProjectUploadGroup[] {
    return Array.from(uploadGroups.value.values()).filter(g => g.failedCount > 0);
  }

  // 清除指定项目的上传记录
  function clearProject(projectId: string): void {
    uploadGroups.value.delete(projectId);
  }

  // 清除所有上传记录
  function clearAll(): void {
    uploadGroups.value.clear();
    activeUploads.value = 0;
  }

  // 增加活跃上传计数
  function incrementActive(): void {
    activeUploads.value++;
  }

  // 减少活跃上传计数
  function decrementActive(): void {
    activeUploads.value = Math.max(0, activeUploads.value - 1);
  }

  return {
    uploadGroups,
    activeUploads,
    addTasks,
    updateTask,
    getFailedTasks,
    getProjectsWithFailures,
    clearProject,
    clearAll,
    incrementActive,
    decrementActive,
  };
});
