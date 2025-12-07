import { invoke } from '@tauri-apps/api/core';
import { ReqToken, ResCaptcha, ResToken } from '../api/model/auth';

export async function getCaptcha(): Promise<ResCaptcha> {
  try {
    // Raw shape from Rust
    interface RawResCaptcha {
      image: string;
      info: string;
    }

    const raw = await invoke<RawResCaptcha>('get_captcha');
    return { image: raw.image, info: raw.info };
  } catch (error) {
    console.error('Error in getCaptcha:', error);
    throw error;
  }
}

export async function aquireMoetranToken(payload: ReqToken): Promise<ResToken> {
  try {
    const body = {
      email: payload.email,
      password: payload.password,
      captcha: payload.captcha,
      captcha_info: payload.info,
    };
    // Raw shape from Rust
    interface RawResToken {
      token: string;
    }

    const raw = await invoke<RawResToken>('aquire_token', { payload: body });
    return { token: raw.token };
  } catch (error) {
    console.error('Error in aquireMoetranToken:', { payload, error });
    throw error;
  }
}

// 获取 Moetran token
export async function getMoetranToken(): Promise<string | null> {
  try {
    return await invoke<string | null>('get_moetran_token');
  } catch (error) {
    console.error('Error in getMoetranToken:', error);
    throw error;
  }
}

// 保存 Moetran token
export async function saveMoetranToken(token: string): Promise<void> {
  try {
    await invoke('save_moetran_token', { token });
  } catch (error) {
    console.error('Error in saveMoetranToken:', { token, error });
    throw error;
  }
}

// 删除 Moetran token
export async function removeMoetranToken(): Promise<void> {
  try {
    await invoke('remove_moetran_token');
  } catch (error) {
    console.error('Error in removeMoetranToken:', error);
    throw error;
  }
}

// 获取 Poprako token
export async function getPoprakoToken(): Promise<string | null> {
  try {
    return await invoke<string | null>('get_poprako_token');
  } catch (error) {
    console.error('Error in getPoprakoToken:', error);
    throw error;
  }
}

// 保存 Poprako token
export async function savePoprakoToken(token: string): Promise<void> {
  try {
    await invoke('save_poprako_token', { token });
  } catch (error) {
    console.error('Error in savePoprakoToken:', { token, error });
    throw error;
  }
}

// 删除 Poprako token
export async function removePoprakoToken(): Promise<void> {
  try {
    await invoke('remove_poprako_token');
  } catch (error) {
    console.error('Error in removePoprakoToken:', error);
    throw error;
  }
}
