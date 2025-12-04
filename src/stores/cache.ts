import { defineStore } from 'pinia';
import { reactive } from 'vue';

interface FileInfo {
  id: string;
  name: string;
  sourceCount: number;
  url: string; // 图片URL（Moetran API 总是返回）
}

export const useCacheStore = defineStore('cache', () => {
  const IMAGE_CACHE_LIMIT = 8;
  const PROJECT_DETAIL_CACHE_LIMIT = 5;

  const state = reactive({
    imageCache: new Map<string, string>(),
    imageFetches: new Map<string, Promise<string>>(),
    projectDetailCache: new Map<string, FileInfo[]>(),
  });

  // Image cache functions
  function promoteImageCacheEntry(key: string): string | null {
    const cached = state.imageCache.get(key);
    if (!cached) return null;
    state.imageCache.delete(key);
    state.imageCache.set(key, cached);
    return cached;
  }

  function enforceImageCacheLimit(): void {
    while (state.imageCache.size > IMAGE_CACHE_LIMIT) {
      const iterator = state.imageCache.keys();
      const oldestKey = iterator.next().value as string | undefined;
      if (!oldestKey) break;
      const oldestUrl = state.imageCache.get(oldestKey);
      if (oldestUrl) {
        try {
          URL.revokeObjectURL(oldestUrl);
        } catch (_) {}
      }
      state.imageCache.delete(oldestKey);
    }
  }

  function storeImageCacheEntry(key: string, url: string): void {
    const existing = state.imageCache.get(key);
    if (existing && existing !== url) {
      try {
        URL.revokeObjectURL(existing);
      } catch (_) {}
    }
    state.imageCache.delete(key);
    state.imageCache.set(key, url);
    enforceImageCacheLimit();
  }

  // Project detail cache functions
  function promoteProjectDetailCacheEntry(key: string): FileInfo[] | null {
    const cached = state.projectDetailCache.get(key);
    if (!cached) return null;
    state.projectDetailCache.delete(key);
    state.projectDetailCache.set(key, cached);
    return cached;
  }

  function enforceProjectDetailCacheLimit(): void {
    while (state.projectDetailCache.size > PROJECT_DETAIL_CACHE_LIMIT) {
      const iterator = state.projectDetailCache.keys();
      const oldestKey = iterator.next().value as string | undefined;
      if (!oldestKey) break;
      state.projectDetailCache.delete(oldestKey);
    }
  }

  function storeProjectDetailCacheEntry(key: string, data: FileInfo[]): void {
    state.projectDetailCache.delete(key);
    state.projectDetailCache.set(key, data);
    enforceProjectDetailCacheLimit();
  }

  return {
    ...state,
    promoteImageCacheEntry,
    enforceImageCacheLimit,
    storeImageCacheEntry,
    promoteProjectDetailCacheEntry,
    enforceProjectDetailCacheLimit,
    storeProjectDetailCacheEntry,
  };
});
