import { invoke } from '@tauri-apps/api/core';
import type { ResCaptcha, ReqMoeToken, ResMoeToken } from './model/auth';

export async function getCaptcha(): Promise<ResCaptcha> {
  return await invoke<ResCaptcha>('get_captcha');
}

export async function aquireMoetranToken(payload: ReqMoeToken): Promise<ResMoeToken> {
  return await invoke<ResMoeToken>('aquire_token', {
    payload: {
      email: payload.email,
      password: payload.password,
      captcha: payload.captcha,
      captcha_info: payload.info,
    },
  });
}
