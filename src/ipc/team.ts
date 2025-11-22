import { invoke } from '@tauri-apps/api/core';
import type { ResTeam } from '../api/model/team';

// 获取当前用户的汉化组列表
export async function getUserTeams(params: { page: number; limit: number }): Promise<ResTeam[]> {
  return await invoke<ResTeam[]>('get_user_teams', params);
}
