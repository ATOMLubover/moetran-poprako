import { invoke } from '@tauri-apps/api/core';
import type { ResTeam } from '../api/model/team';

// 获取当前用户的汉化组列表
export async function getUserTeams(params: { page: number; limit: number }): Promise<ResTeam[]> {
  // Raw response shape from Moetran (snake_case)
  interface RawResTeam {
    id: string;
    avatar: string;
    has_avatar: boolean;
    name: string;
    // optional fields from backend may include is_admin etc.
    [key: string]: unknown;
  }

  const raw = await invoke<RawResTeam[]>('get_user_teams', {
    payload: { page: params.page, limit: params.limit },
  });

  return (raw || []).map(r => ({
    id: r.id,
    avatar: r.avatar,
    hasAvatar: !!r.has_avatar,
    name: r.name,
  }));
}
