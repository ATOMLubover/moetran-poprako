import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getTeamPoprakoProjsets, type PoprakoProjsetInfo } from '../ipc/project';

// 缓存每个团队下的 PopRaKo 项目集列表，供创建项目时选择
export const useProjsetStore = defineStore('projset', () => {
  const projsetsByTeam = ref<Record<string, PoprakoProjsetInfo[]>>({});
  const loading = ref(false);

  async function loadForTeam(teamId: string): Promise<void> {
    if (!teamId) return;
    loading.value = true;
    try {
      const list = await getTeamPoprakoProjsets(teamId);
      projsetsByTeam.value[teamId] = list;
    } finally {
      loading.value = false;
    }
  }

  function getForTeam(teamId: string): PoprakoProjsetInfo[] {
    return projsetsByTeam.value[teamId] ?? [];
  }

  return {
    projsetsByTeam,
    loading,
    loadForTeam,
    getForTeam,
  };
});
