// 图片缓存相关 IPC 调用
import { invoke } from '@tauri-apps/api/core';

export interface FileDownloadInfo {
  url: string;
}

export interface CachedFileData {
  b64: string;
  content_type: string;
}

export interface CachedProjectMetadata {
  projectId: string;
  projectName: string;
  status: string; // "completed" | "failed"
  fileCount: number;
  totalSizeBytes: number;
  cachedAt: number; // Unix timestamp
}

// Raw snake_case 结构体，用于接收 Rust 后端数据
interface RawCachedProjectMetadata {
  project_id: string;
  project_name: string;
  status: string;
  file_count: number;
  total_size_bytes: number;
  cached_at: number;
}

/**
 * 检查项目的图片缓存是否存在
 */
export async function checkFileCache(projectId: string): Promise<boolean> {
  return await invoke<boolean>('check_file_cache', {
    projectId,
  });
}

/**
 * 下载整个项目的所有图片到本地缓存
 */
export async function downloadProjectFiles(
  projectId: string,
  projectName: string,
  files: FileDownloadInfo[]
): Promise<void> {
  await invoke('download_project_files', {
    projectId,
    projectName,
    files,
  });
}

/**
 * 删除项目的图片缓存
 */
export async function deleteFileCache(projectId: string): Promise<void> {
  await invoke('delete_file_cache', {
    projectId,
  });
}

/**
 * 从本地缓存读取图片（base64 编码）
 */
export async function loadCachedFile(
  projectId: string,
  fileIndex: number
): Promise<CachedFileData> {
  return await invoke<CachedFileData>('load_cached_file', {
    projectId,
    fileIndex,
  });
}

/**
 * 获取所有缓存项目列表
 */
export async function getAllCachedProjects(): Promise<CachedProjectMetadata[]> {
  const raw = await invoke<RawCachedProjectMetadata[]>('get_all_cached_projects_list');

  return (raw || []).map(r => ({
    projectId: r.project_id,
    projectName: r.project_name,
    status: r.status,
    fileCount: r.file_count,
    totalSizeBytes: r.total_size_bytes,
    cachedAt: r.cached_at,
  }));
}

/**
 * 获取单个项目的缓存元数据
 */
export async function getCachedProjectInfo(
  projectId: string
): Promise<CachedProjectMetadata | null> {
  return await invoke<CachedProjectMetadata | null>('get_cached_project_info', {
    projectId,
  });
}
