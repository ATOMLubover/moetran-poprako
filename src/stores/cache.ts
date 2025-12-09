import { defineStore } from 'pinia';
import { reactive } from 'vue';

export const useCacheStore = defineStore('cache', () => {
  const IMAGE_CACHE_LIMIT = 8;

  const state = reactive({
    imageCache: new Map<string, string>(),
    imageFetches: new Map<string, Promise<string>>(),
    // 项目文件缓存状态：projectId -> boolean (是否已缓存到磁盘)
    projectFileCacheState: new Map<string, boolean>(),
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

  // Project detail cache functions - removed: should not cache content other than images
  // function promoteProjectDetailCacheEntry(key: string): FileInfo[] | null {
  //   const cached = state.projectDetailCache.get(key);
  //   if (!cached) return null;
  //   state.projectDetailCache.delete(key);
  //   state.projectDetailCache.set(key, cached);
  //   return cached;
  // }

  // function enforceProjectDetailCacheLimit(): void {
  //   while (state.projectDetailCache.size > PROJECT_DETAIL_CACHE_LIMIT) {
  //     const iterator = state.projectDetailCache.keys();
  //     const oldestKey = iterator.next().value as string | undefined;
  //     if (!oldestKey) break;
  //     state.projectDetailCache.delete(oldestKey);
  //   }
  // }

  // function storeProjectDetailCacheEntry(key: string, data: FileInfo[]): void {
  //   state.projectDetailCache.delete(key);
  //   state.projectDetailCache.set(key, data);
  //   enforceProjectDetailCacheLimit();
  // }

  // Project file cache state functions
  function setProjectFileCacheState(projectId: string, hasCached: boolean): void {
    state.projectFileCacheState.set(projectId, hasCached);
  }

  function getProjectFileCacheState(projectId: string): boolean | undefined {
    return state.projectFileCacheState.get(projectId);
  }

  return {
    ...state,
    promoteImageCacheEntry,
    enforceImageCacheLimit,
    storeImageCacheEntry,
    setProjectFileCacheState,
    getProjectFileCacheState,
  };
});
