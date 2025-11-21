import { invoke } from '@tauri-apps/api/core';
import { ResMoeToken } from '../api/model/auth';

export async function getCaptcha(): Promise<ResMoeToken> {
  // Throw error to caller.
  let response = await invoke<ResMoeToken>('get_captcha');

  return response;
}

export async function __mockStoreMoetranToken(token: string) {
  // TODO
  console.log('Mock storing Moetran token:', token);
}
