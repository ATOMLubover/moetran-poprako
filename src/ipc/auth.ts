import { invoke } from '@tauri-apps/api/core';
import { UserLoginResp } from '../api/model/auth';

export async function getCaptcha(): Promise<UserLoginResp> {
  // Throw error to caller.
  let response = await invoke<UserLoginResp>('get_captcha');

  return response;
}

export async function __mockStoreMoetranToken(token: string) {
  // TODO
  console.log('Mock storing Moetran token:', token);
}
