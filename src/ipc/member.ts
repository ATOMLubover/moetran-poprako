import { invoke } from '@tauri-apps/api/core';

export interface ResMemberBrief {
  memberId: string;
  username: string;
}

export interface ResMemberInfo {
  memberId: string;
  isAdmin: boolean;
  isTranslator: boolean;
  isProofreader: boolean;
  isTypesetter: boolean;
  isPrincipal: boolean;
}

export type MemberPosition = 'translator' | 'proofreader' | 'typesetter' | 'principal';

export interface GetMembersParams {
  teamId: string;
  position?: MemberPosition;
  fuzzyName?: string;
  page?: number;
  limit?: number;
}

// legacy: MembersReply removed in favor of RawMembersReply mapping
// Raw interfaces from Rust (snake_case)
interface RawResMemberBrief {
  member_id: string;
  username: string;
}

interface RawMembersReply {
  items: RawResMemberBrief[];
}

export async function searchMembersByName(params: GetMembersParams): Promise<ResMemberBrief[]> {
  // Tauri command `get_members` expects a single argument named `payload` (the ReqMembers struct).
  // Pass the params wrapped under `payload` so argument names match the Rust command signature.
  // Note: the Rust `ReqMembers` struct uses snake_case field names (e.g. `team_id`).
  // Tauri maps top-level arg names (e.g. `teamId` -> `team_id`) but does NOT
  // automatically convert nested object keys. Convert the inner keys to snake_case
  // so Serde can deserialize into `ReqMembers` correctly.
  const payload = {
    team_id: params.teamId,
    position: params.position,
    fuzzy_name: params.fuzzyName,
    page: params.page,
    limit: params.limit,
  };

  const raw = await invoke<RawMembersReply>('get_members', { payload });
  return (raw.items || []).map(i => ({ memberId: i.member_id, username: i.username }));
}

// 获取当前登录用户在指定 team 中的成员信息（含 is_admin 标记）
interface RawMemberInfo {
  member_id: string;
  is_admin?: boolean;
  is_translator?: boolean;
  is_proofreader?: boolean;
  is_typesetter?: boolean;
  is_principal?: boolean;
}

export async function getMemberInfo(teamId: string): Promise<ResMemberInfo> {
  const raw = await invoke<RawMemberInfo>('get_member_info', { payload: { team_id: teamId } });
  return {
    memberId: raw.member_id,
    isAdmin: raw.is_admin === true,
    isTranslator: raw.is_translator === true,
    isProofreader: raw.is_proofreader === true,
    isTypesetter: raw.is_typesetter === true,
    isPrincipal: raw.is_principal === true,
  };
}
