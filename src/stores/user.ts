import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { ResUser } from '../api/model/user';

// 用户信息与在各团队的管理员标记
export const useUserStore = defineStore('user', () => {
  const user = ref<ResUser | null>(null);

  // teamId -> isAdmin | undefined (undefined = unknown)
  const adminByTeam = ref<Record<string, boolean | undefined>>({});

  function setUser(u: ResUser | null) {
    user.value = u;
  }

  function setAdminForTeam(teamId: string, isAdmin: boolean) {
    adminByTeam.value[teamId] = isAdmin;
  }

  function isAdminFor(teamId?: string | null): boolean {
    if (!teamId) return false;
    const v = adminByTeam.value[teamId];
    // 默认认为未知即非管理员，依赖显式的 member/info 接口刷新
    return v === true;
  }

  return {
    user,
    adminByTeam,
    setUser,
    setAdminForTeam,
    isAdminFor,
  };
});
