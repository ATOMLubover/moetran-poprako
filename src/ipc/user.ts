import { invoke } from '@tauri-apps/api/core';
import type { ResSync, ResUser } from '../api/model/user';

// PopRaKo 用户同步（使用 email 作为后端的 username 字段）
export async function syncUser(payload: {
  userId: string;
  username: string;
  email: string;
}): Promise<ResSync> {
  const body = {
    user_id: payload.userId,
    username: payload.username,
    email: payload.email,
  };

  // Raw response shape from PopRaKo (snake_case)
  interface RawResSync {
    token: string;
  }

  const raw = await invoke<RawResSync>('sync_user', { payload: body });
  return { token: raw.token };
}

// 获取用户信息
export async function getUserInfo(): Promise<ResUser> {
  // Raw response shape from Moetran (snake_case)
  interface RawResUser {
    id: string;
    name: string;
    has_avatar: boolean;
    avatar: string;
  }

  const raw = await invoke<RawResUser>('get_user_info');
  return {
    id: raw.id,
    name: raw.name,
    hasAvatar: raw.has_avatar,
    avatar: raw.avatar,
  };
}

// (用户汉化组与项目相关接口已迁移到 team.ts / project.ts)
