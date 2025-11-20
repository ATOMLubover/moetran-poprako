import { invoke } from '@tauri-apps/api/core';
import type { GetCaptchaResp, UserLoginReq, UserLoginResp } from './model/auth';

export async function getCaptcha(): Promise<GetCaptchaResp> {
  // Tauri backend proxy to avoid CORS
  return (await invoke('get_captcha')) as GetCaptchaResp;
}

export async function requestLoginToken(payload: UserLoginReq): Promise<UserLoginResp> {
  const result = (await invoke('request_token', {
    payload: {
      email: payload.email,
      password: payload.password,
      captcha: payload.captcha,
      captcha_info: payload.info,
    },
  })) as { token: string };

  return { token: result.token } satisfies UserLoginResp;
}
