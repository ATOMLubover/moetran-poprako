import { invoke } from '@tauri-apps/api/core';

export interface ResMemberBrief {
  member_id: string;
  username: string;
}

export type MemberPosition = 'translator' | 'proofreader' | 'typesetter' | 'principal';

export interface GetMembersParams {
  team_id: string;
  position?: MemberPosition;
  fuzzy_name?: string;
  page?: number;
  limit?: number;
}

interface MembersReply {
  items: ResMemberBrief[];
}

export async function searchMembersByName(params: GetMembersParams): Promise<ResMemberBrief[]> {
  const result = await invoke<MembersReply>('get_members', params);
  return result.items;
}
