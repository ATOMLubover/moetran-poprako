import { invoke } from '@tauri-apps/api/core';

/**
 * 检查应用是否有更新
 * @returns 如果有更新返回true，否则false
 */
export async function checkAppUpdate(): Promise<boolean> {
  try {
    return await invoke<boolean>('update');
  } catch (error) {
    console.error('Error in checkAppUpdate:', error);
    throw error;
  }
}
