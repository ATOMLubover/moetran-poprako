import { invoke } from '@tauri-apps/api/core';
import type { ResLogin, ResUser } from '../api/model/user';

// PopRaKo 用户同步（使用 email 作为后端的 username 字段）
export async function syncUser(payload: {
  user_id: string;
  username: string;
  email: string;
  password: string;
}): Promise<ResLogin> {
  return await invoke<ResLogin>('sync_user', { payload });
}

// 获取用户信息
export async function getUserInfo(): Promise<ResUser> {
  return await invoke<ResUser>('get_user_info');
}

// (用户汉化组与项目相关接口已迁移到 team.ts / project.ts)
