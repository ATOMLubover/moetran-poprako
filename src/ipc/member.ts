import { invoke } from '@tauri-apps/api/core';
import type { ResMemberBrief, ResMemberInfo } from '../api/model/member';

export type MemberPosition = 'translator' | 'proofreader' | 'typesetter' | 'principal' | 'redrawer';

export interface GetMembersParams {
  teamId: string;
  position?: MemberPosition;
  fuzzyName?: string;
  page?: number;
  limit?: number;
}

// Note: backend serializes using camelCase; invoke return type uses DTOs from api/model

export async function searchMembersByName(params: GetMembersParams): Promise<ResMemberBrief[]> {
  try {
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

    // Raw response from PopRaKo: items with snake_case keys
    interface RawResMemberBrief {
      member_id: string;
      username: string;
    }

    interface RawMembersReply {
      items: RawResMemberBrief[];
    }

    const raw = await invoke<RawMembersReply>('get_members', { payload });
    return (raw.items || []).map(i => ({ memberId: i.member_id, username: i.username }));
  } catch (error) {
    console.error('Error in searchMembersByName:', { params, error });
    throw error;
  }
}

// 获取当前登录用户在指定 team 中的成员信息（含 is_admin 标记）
export async function getMemberInfo(teamId: string): Promise<ResMemberInfo> {
  try {
    // Raw response shape from PopRaKo (snake_case)
    interface RawMemberInfo {
      member_id: string;
      is_admin?: boolean;
      is_translator?: boolean;
      is_proofreader?: boolean;
      is_typesetter?: boolean;
      is_principal?: boolean;
      is_redrawer?: boolean;
    }

    const raw = await invoke<RawMemberInfo>('get_member_info', { payload: { team_id: teamId } });
    return {
      memberId: raw.member_id,
      isAdmin: raw.is_admin === true,
      isTranslator: raw.is_translator === true,
      isProofreader: raw.is_proofreader === true,
      isTypesetter: raw.is_typesetter === true,
      isPrincipal: raw.is_principal === true,
      isRedrawer: raw.is_redrawer === true,
    };
  } catch (error) {
    console.error('Error in getMemberInfo:', { teamId, error });
    throw error;
  }
}

// 获取团队中所有成员及其最后活跃时间
export async function getActiveMembers(
  teamId: string,
  page: number = 1,
  limit: number = 15
): Promise<ResMember[]> {
  try {
    // Raw response shape from PopRaKo (snake_case)
    interface RawActiveMember {
      member_id: string;
      user_id: string;
      username: string;
      is_admin?: boolean;
      is_translator?: boolean;
      is_proofreader?: boolean;
      is_typesetter?: boolean;
      is_redrawer?: boolean;
      is_principal?: boolean;
      // unix timestamp in seconds
      last_active?: number | null;
    }

    const raw = await invoke<RawActiveMember[]>('get_active_members', {
      payload: {
        team_id: teamId,
        page,
        limit,
      },
    });

    return raw.map(m => ({
      userId: m.user_id,
      memberId: m.member_id,
      username: m.username,
      isAdmin: m.is_admin === true,
      isTranslator: m.is_translator === true,
      isProofreader: m.is_proofreader === true,
      isTypesetter: m.is_typesetter === true,
      isRedrawer: m.is_redrawer === true,
      isPrincipal: m.is_principal === true,
      lastActive: m.last_active ?? null,
    }));
  } catch (error) {
    console.error('Error in getActiveMembers:', { teamId, page, limit, error });
    throw error;
  }
}
