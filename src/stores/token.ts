import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  getMoetranToken,
  getPoprakoToken,
  saveMoetranToken,
  savePoprakoToken,
  removeMoetranToken,
  removePoprakoToken,
} from '../ipc/auth';

// 全局 token 状态使用组合式 API，避免 this 类型推断问题
export const useTokenStore = defineStore('token', () => {
  // Moetran Token
  const moetranToken = ref<string | null>(null);
  // Poprako Token
  const poprakoToken = ref<string | null>(null);

  // 加载两个 token（后端失败则保持现有副本）
  async function loadAll(): Promise<void> {
    try {
      moetranToken.value = await getMoetranToken();
    } catch (e) {
      console.warn('加载 Moetran token 失败', e);
      if (!moetranToken.value) {
        const needRelogin = window.confirm('后端读取 Moetran Token 失败，是否重新登录？');
        if (needRelogin) {
          // 清空前端副本，交给登录页处理重新登录
          moetranToken.value = null;
        }
      }
    }

    try {
      poprakoToken.value = await getPoprakoToken();
    } catch (e) {
      console.warn('加载 Poprako token 失败', e);
    }
  }

  // 设置 Moetran Token（前端优先，后端失败保留副本）
  async function setMoetranToken(token: string): Promise<void> {
    moetranToken.value = token;
    try {
      await saveMoetranToken(token);
    } catch (e) {
      console.warn('保存 Moetran token 到后端失败', e);
    }
  }

  // 设置 Poprako Token
  async function setPoprakoToken(token: string): Promise<void> {
    poprakoToken.value = token;
    try {
      await savePoprakoToken(token);
    } catch (e) {
      console.warn('保存 Poprako token 到后端失败', e);
    }
  }

  // 清除所有 Token
  async function clearAll(): Promise<void> {
    moetranToken.value = null;
    poprakoToken.value = null;
    try {
      await removeMoetranToken();
    } catch {}
    try {
      await removePoprakoToken();
    } catch {}
  }

  return { moetranToken, poprakoToken, loadAll, setMoetranToken, setPoprakoToken, clearAll };
});
