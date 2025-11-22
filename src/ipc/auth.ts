import { invoke } from '@tauri-apps/api/core';
import { ReqToken, ResCaptcha, ResToken } from '../api/model/auth';

export async function getCaptcha(): Promise<ResCaptcha> {
  return await invoke<ResCaptcha>('get_captcha');
}

export async function aquireMoetranToken(payload: ReqToken): Promise<ResToken> {
  return await invoke<ResToken>('aquire_token', {
    payload: {
      email: payload.email,
      password: payload.password,
      captcha: payload.captcha,
      captcha_info: payload.info,
    },
  });
}

// 获取 Moetran token
export async function getMoetranToken(): Promise<string | null> {
  return await invoke<string | null>('get_moetran_token');
}

// 保存 Moetran token
export async function saveMoetranToken(token: string): Promise<void> {
  await invoke('save_moetran_token', { token });
}

// 删除 Moetran token
export async function removeMoetranToken(): Promise<void> {
  await invoke('remove_moetran_token');
}

// 获取 Poprako token
export async function getPoprakoToken(): Promise<string | null> {
  return await invoke<string | null>('get_poprako_token');
}

// 保存 Poprako token
export async function savePoprakoToken(token: string): Promise<void> {
  await invoke('save_poprako_token', { token });
}

// 删除 Poprako token
export async function removePoprakoToken(): Promise<void> {
  await invoke('remove_poprako_token');
}
