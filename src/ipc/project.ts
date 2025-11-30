import { invoke } from '@tauri-apps/api/core';
import type { ResProjectEnriched } from '../api/model/project';
import type { ResMember } from '../api/model/member';
import type { ResTeam } from '../api/model/team';

// Private raw (snake_case) interfaces from Rust/PopRaKo responses
interface RawPoprakoMember {
  user_id?: string;
  member_id: string;
  username: string;
  is_admin?: boolean;
  is_translator?: boolean;
  is_proofreader?: boolean;
  is_typesetter?: boolean;
  is_principal?: boolean;
}

interface RawResProject {
  id: string;
  name: string;
  source_count: number;
  translated_source_count: number;
  checked_source_count: number;
  team: any;
  project_set?: { id: string; name: string } | null;
  has_poprako: boolean;
  projset_index?: number | null;
  translating_status?: number | null;
  proofreading_status?: number | null;
  typesetting_status?: number | null;
  reviewing_status?: number | null;
  is_published?: boolean | null;
  members?: RawPoprakoMember[] | null;
  principals?: string[] | null;
}

function mapRawTeam(t: any): ResTeam {
  if (!t) return { id: '', avatar: '', hasAvatar: false, name: '' };
  return {
    id: t.id,
    avatar: t.avatar,
    hasAvatar: typeof t.has_avatar !== 'undefined' ? !!t.has_avatar : !!t.hasAvatar,
    name: t.name,
  };
}

function mapRawMember(m: RawPoprakoMember): ResMember {
  return {
    userId: m.user_id ?? (m as any).userId ?? '',
    memberId: m.member_id,
    username: m.username,
    isAdmin: !!m.is_admin,
    isTranslator: !!m.is_translator,
    isProofreader: !!m.is_proofreader,
    isTypesetter: !!m.is_typesetter,
    isPrincipal: !!m.is_principal,
  };
}

function mapRawProject(r: RawResProject): ResProjectEnriched {
  console.log('Mapping RawResProject to ResProjectEnriched', r);

  return {
    id: r.id,
    name: r.name,
    sourceCount: r.source_count,
    translatedSourceCount: r.translated_source_count,
    checkedSourceCount: r.checked_source_count,
    team: mapRawTeam(r.team) as any,
    projectSet: r.project_set
      ? { id: r.project_set.id, name: r.project_set.name }
      : (undefined as any),
    hasPoprako: r.has_poprako,
    projsetIndex: r.projset_index ?? undefined,
    translatingStatus: r.translating_status ?? undefined,
    proofreadingStatus: r.proofreading_status ?? undefined,
    typesettingStatus: r.typesetting_status ?? undefined,
    reviewingStatus: r.reviewing_status ?? undefined,
    isPublished: typeof r.is_published === 'boolean' ? r.is_published : undefined,
    members: (r.members || []).map(m => mapRawMember(m)) as ResMember[],
    principals:
      r.principals ??
      ((r.members || []) as RawPoprakoMember[])
        .filter(m => m.is_principal)
        .map(m => m.user_id ?? m.member_id),
  } as ResProjectEnriched;
}

// Raw PopRaKo member shape as returned from backend (snake_case).
// NOTE: backend now serializes DTOs using `camelCase` (serde rename_all).
// Use the frontend DTOs from `src/api/model` as the invoke return types.

// 获取当前用户的 enriched 项目列表
export async function getUserProjectsEnriched(params: {
  page: number;
  limit: number;
}): Promise<ResProjectEnriched[]> {
  console.log('Invoking getUserProjectsEnriched with params', params);

  const raw = await invoke<RawResProject[]>('get_user_projects_enriched', {
    payload: {
      page: params.page,
      limit: params.limit,
    },
  });

  return (raw || []).map(r => mapRawProject(r));
}

// PopRaKo 主导的项目搜索参数（与 PoprakoProjFilterReq 对应的前端版本）
export interface ProjectSearchFilters {
  fuzzyProjName?: string;
  translatingStatus?: number;
  proofreadingStatus?: number;
  typesettingStatus?: number;
  reviewingStatus?: number;
  isPublished?: boolean;
  memberIds?: string[];
  projsetIds?: string[];
  timeStart?: number;
  page?: number;
  limit?: number;
  [key: string]: unknown;
}

// 基于 PopRaKo /projs/search + Moetran /user/projects?word= 的用户项目搜索
export async function searchUserProjectsEnriched(
  filters: ProjectSearchFilters
): Promise<ResProjectEnriched[]> {
  const payload = {
    fuzzy_proj_name: filters.fuzzyProjName,
    translating_status: filters.translatingStatus,
    proofreading_status: filters.proofreadingStatus,
    typesetting_status: filters.typesettingStatus,
    reviewing_status: filters.reviewingStatus,
    is_published: filters.isPublished,
    member_ids: filters.memberIds,
    projset_ids: filters.projsetIds,
    time_start: filters.timeStart,
    page: filters.page,
    limit: filters.limit,
  };
  // Rust expects a single argument named `filter: PoprakoProjFilterReq`,
  // so pass the snake_cased object as the top-level `filter` key.
  const raw = await invoke<RawResProject[]>('search_user_projects_enriched', {
    filter: payload,
  });

  return (raw || []).map(r => mapRawProject(r));
}

// 基于 PopRaKo /projs/search + Moetran /teams/:team_id/projects?word= 的团队项目搜索
export async function searchTeamProjectsEnriched(
  params: {
    team_id: string;
  } & ProjectSearchFilters
): Promise<ResProjectEnriched[]> {
  const payload = {
    fuzzy_proj_name: params.fuzzyProjName,
    translating_status: params.translatingStatus,
    proofreading_status: params.proofreadingStatus,
    typesetting_status: params.typesettingStatus,
    reviewing_status: params.reviewingStatus,
    is_published: params.isPublished,
    member_ids: params.memberIds,
    projset_ids: params.projsetIds,
    time_start: params.timeStart,
    page: params.page,
    limit: params.limit,
  };
  // Pass a single `payload` object with `team_id` and nested `filter` (snake_case).
  const raw = await invoke<RawResProject[]>('search_team_projects_enriched', {
    payload: {
      team_id: params.team_id,
      filter: {
        fuzzy_proj_name: payload.fuzzy_proj_name,
        translating_status: payload.translating_status,
        proofreading_status: payload.proofreading_status,
        typesetting_status: payload.typesetting_status,
        reviewing_status: payload.reviewing_status,
        is_published: payload.is_published,
        member_ids: payload.member_ids,
        time_start: payload.time_start,
        page: payload.page,
        limit: payload.limit,
      },
    },
  });

  return (raw || []).map(r => mapRawProject(r));
}

// 获取团队的 enriched 项目列表（无筛选，分页）
export async function getTeamProjectsEnriched(params: {
  teamId: string;
  page: number;
  limit: number;
}): Promise<ResProjectEnriched[]> {
  const raw = await invoke<RawResProject[]>('get_team_projects_enriched', {
    payload: {
      team_id: params.teamId,
      page: params.page,
      limit: params.limit,
    },
  });

  return (raw || []).map(r => mapRawProject(r));
}

// PopRaKo 创建项目集请求参数
export interface CreateProjsetPayload {
  projsetName: string;
  projsetDescription: string;
  teamId: string;
  mtrToken: string;
}

export interface CreateProjsetResult {
  projsetSerial: number;
}

// 调用 PopRaKo /projset/create 的 IPC 封装
// Raw shape from Rust (snake_case)
interface RawCreateProjsetResult {
  projset_serial: number;
}

export async function createProjset(payload: CreateProjsetPayload): Promise<CreateProjsetResult> {
  const raw = await invoke<RawCreateProjsetResult>('create_projset', {
    payload: {
      projset_name: payload.projsetName,
      projset_description: payload.projsetDescription,
      team_id: payload.teamId,
      mtr_token: payload.mtrToken,
    },
  });

  return { projsetSerial: raw.projset_serial };
}

// PopRaKo 创建项目请求参数
export interface CreateProjPayload {
  projName: string;
  projDescription: string;
  teamId: string;
  projsetId: string;
  mtrAuth: string;
  worksetIndex: number;
  sourceLanguage: string;
  targetLanguages: string[];
  allowApplyType: number;
  applicationCheckType: number;
  defaultRole: string;
}

export interface CreateProjResult {
  projId: string;
  projSerial: number;
  projsetIndex: number;
}

// 调用 PopRaKo /proj/create 的 IPC 封装
interface RawCreateProjResult {
  proj_id: string;
  proj_serial: number;
  projset_index: number;
}

export async function createProj(payload: CreateProjPayload): Promise<CreateProjResult> {
  const raw = await invoke<RawCreateProjResult>('create_proj', {
    payload: {
      proj_name: payload.projName,
      proj_description: payload.projDescription,
      team_id: payload.teamId,
      projset_id: payload.projsetId,
      mtr_auth: payload.mtrAuth,
      workset_index: payload.worksetIndex,
      source_language: payload.sourceLanguage,
      target_languages: payload.targetLanguages,
      allow_apply_type: payload.allowApplyType,
      application_check_type: payload.applicationCheckType,
      default_role: payload.defaultRole,
    },
  });

  return {
    projId: raw.proj_id,
    projSerial: raw.proj_serial,
    projsetIndex: raw.projset_index,
  };
}

// PopRaKo 项目集 DTO（简化版，只保留 Creator 需要的字段）
export interface PoprakoProjsetInfo {
  projsetId: string;
  projsetName: string;
  projsetDescription?: string | null;
  projsetSerial: number;
  teamId: string;
}

// 获取 PopRaKo 中指定团队下的项目集列表
interface RawPoprakoProjsetInfo {
  projset_id: string;
  projset_name: string;
  projset_description?: string | null;
  projset_serial: number;
  team_id: string;
}

export async function getTeamPoprakoProjsets(teamId: string): Promise<PoprakoProjsetInfo[]> {
  const raw = await invoke<RawPoprakoProjsetInfo[]>('get_team_poprako_projsets', {
    payload: { team_id: teamId },
  });

  return (raw || []).map(r => ({
    projsetId: r.projset_id,
    projsetName: r.projset_name,
    projsetDescription: r.projset_description ?? null,
    projsetSerial: r.projset_serial,
    teamId: r.team_id,
  }));
}

// 指派成员到 PopRaKo 项目
export interface AssignMemberPayload {
  projId: string;
  memberId: string;
  isTranslator: boolean;
  isProofreader: boolean;
  isTypesetter: boolean;
  isPrincipal: boolean;
}

export async function assignMemberToProj(payload: AssignMemberPayload): Promise<void> {
  await invoke<void>('assign_member_to_proj', {
    payload: {
      proj_id: payload.projId,
      member_id: payload.memberId,
      is_translator: payload.isTranslator,
      is_proofreader: payload.isProofreader,
      is_typesetter: payload.isTypesetter,
      is_principal: payload.isPrincipal,
    },
  });
}

// ========== Moetran 项目 targets / files（供 ProjectDetail 使用） ==========

export interface ProjectTargetInfo {
  id: string;
  translatedSourceCount: number;
  checkedSourceCount: number;
}

export interface ProjectFileInfo {
  id: string;
  name: string;
  sourceCount: number;
}

export async function getProjectTargets(projectId: string): Promise<ProjectTargetInfo[]> {
  try {
    console.debug('[ipc] invoke get_project_targets', { projectId });
    const raw = await invoke<
      {
        id: string;
        translated_source_count: number;
        checked_source_count: number;
      }[]
    >('get_project_targets', {
      payload: { project_id: projectId },
    });

    console.debug('[ipc] get_project_targets result', { projectId, raw });

    return (raw || []).map(t => ({
      id: t.id,
      translatedSourceCount: t.translated_source_count ?? 0,
      checkedSourceCount: t.checked_source_count ?? 0,
    }));
  } catch (err) {
    console.error('[ipc] getProjectTargets failed', { projectId, err });
    throw err;
  }
}

export async function getProjectFiles(
  projectId: string,
  targetId?: string
): Promise<ProjectFileInfo[]> {
  try {
    console.debug('[ipc] invoke get_project_files', { projectId, targetId });
    const payload: Record<string, string | undefined> = { project_id: projectId };
    if (targetId) payload.target_id = targetId;

    const raw = await invoke<
      {
        id: string;
        name: string;
        source_count: number;
      }[]
    >('get_project_files', {
      payload,
    });

    console.debug('[ipc] get_project_files result', { projectId, targetId, raw });

    return (raw || []).map(f => ({
      id: f.id,
      name: f.name,
      sourceCount: f.source_count ?? 0,
    }));
  } catch (err) {
    console.error('[ipc] getProjectFiles failed', { projectId, targetId, err });
    throw err;
  }
}
