import { invoke } from '@tauri-apps/api/core';
import type { ResProjectEnriched } from '../api/model/project';
import type { ResMember } from '../api/model/member';
import type { ResTeam } from '../api/model/team';
import type { ResAssignment } from '../api/model/assignment';

// Private raw (snake_case) interfaces from Rust/PopRaKo responses
interface RawPoprakoMember {
  user_id?: string;
  member_id: string;
  username: string;
  is_admin?: boolean;
  is_translator?: boolean;
  is_proofreader?: boolean;
  is_typesetter?: boolean;
  is_redrawer?: boolean;
  is_principal?: boolean;
}

interface RawResProject {
  id: string;
  name: string;
  source_count: number;
  translated_source_count: number;
  checked_source_count: number;
  team?: RawTeam | null;
  project_set?: RawProjectSet | null;
  has_poprako: boolean;
  projset_index?: number | null;
  translating_status?: number | null;
  proofreading_status?: number | null;
  typesetting_status?: number | null;
  reviewing_status?: number | null;
  is_published?: boolean | null;
  members?: RawPoprakoMember[] | null;
  principals?: string[] | null;
  // Moetran 原生项目可能返回的 role 字段（object | null）
  role?: RawProjectRole | null;
}

// 私有类型：Raw team shape from backend (snake_case or camelCase tolerant)
interface RawTeam {
  id: string;
  avatar?: string;
  has_avatar?: boolean;
  hasAvatar?: boolean;
  name?: string;
}

// 私有类型：raw project_set shape
interface RawProjectSet {
  id: string;
  name: string;
}

// 私有类型：project role passthrough（结构不确定；使用索引签名替代 any）
interface RawProjectRole {
  [key: string]: unknown;
}

function mapRawTeam(t?: RawTeam | null): ResTeam {
  if (!t) return { id: '', avatar: '', hasAvatar: false, name: '' };
  return {
    id: t.id,
    avatar: t.avatar ?? '',
    hasAvatar: typeof t.has_avatar !== 'undefined' ? !!t.has_avatar : !!t.hasAvatar,
    name: t.name ?? '',
  };
}

function mapRawMember(m: RawPoprakoMember): ResMember {
  return {
    userId: m.user_id ?? '',
    memberId: m.member_id,
    username: m.username,
    isAdmin: !!m.is_admin,
    isTranslator: !!m.is_translator,
    isProofreader: !!m.is_proofreader,
    isTypesetter: !!m.is_typesetter,
    isRedrawer: !!m.is_redrawer,
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
    team: mapRawTeam(r.team),
    projectSet: r.project_set ? { id: r.project_set.id, name: r.project_set.name } : undefined,
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
    // passthrough Moetran `role` for native projects; frontend will only check null/non-null
    role: r.role ?? null,
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

// 创建 PopRaKo 项目集
export interface CreatePoprakoProjsetPayload {
  projsetName: string;
  projsetDescription?: string;
  teamId: string;
  mtrToken: string;
}

export async function createPoprakoProjset(
  payload: CreatePoprakoProjsetPayload
): Promise<{ projsetSerial: number }> {
  const raw = await invoke<{ projset_serial: number }>('create_poprako_projset', {
    payload: {
      projset_name: payload.projsetName,
      projset_description: payload.projsetDescription ?? null,
      team_id: payload.teamId,
      mtr_token: payload.mtrToken,
    },
  });

  return {
    projsetSerial: raw.projset_serial,
  };
}

// 指派成员到 PopRaKo 项目
export interface AssignMemberPayload {
  projId: string;
  memberId: string;
  isTranslator: boolean;
  isProofreader: boolean;
  isTypesetter: boolean;
  isRedrawer: boolean;
}

export async function assignMemberToProj(payload: AssignMemberPayload): Promise<void> {
  await invoke<void>('assign_member_to_proj', {
    payload: {
      proj_id: payload.projId,
      member_id: payload.memberId,
      is_translator: payload.isTranslator,
      is_proofreader: payload.isProofreader,
      is_typesetter: payload.isTypesetter,
      is_redrawer: payload.isRedrawer,
    },
  });
}

// Update project phase status (PopRaKo API #9)
export interface UpdateProjStatusPayload {
  projId: string;
  statusType: 'translating' | 'proofreading' | 'typesetting' | 'reviewing';
  newStatus: number; // 0=pending, 1=wip, 2=completed
}

export async function updateProjStatus(payload: UpdateProjStatusPayload): Promise<void> {
  await invoke<void>('update_proj_status', {
    payload: {
      proj_id: payload.projId,
      status_type: payload.statusType,
      new_status: payload.newStatus,
    },
  });
}

// Publish project (PopRaKo API #10)
export interface PublishProjPayload {
  projId: string;
}

export async function publishProj(payload: PublishProjPayload): Promise<void> {
  await invoke<void>('publish_proj', {
    payload: {
      proj_id: payload.projId,
    },
  });
}

// 获取 assignments（派活列表）
interface RawResAssignment {
  proj_id: string;
  proj_name: string;
  projset_serial: number;
  projset_index: number;
  member_id: string;
  username: string;
  is_translator: boolean;
  is_proofreader: boolean;
  is_typesetter: boolean;
  is_redrawer: boolean;
  updated_at: number;
}

export async function getAssignments(timeStart?: number): Promise<ResAssignment[]> {
  const raw = await invoke<RawResAssignment[]>('get_assignments', {
    payload: {
      time_start: timeStart ?? 0,
    },
  });

  return (raw || []).map(r => ({
    projId: r.proj_id,
    projName: r.proj_name,
    projsetSerial: r.projset_serial,
    projsetIndex: r.projset_index,
    memberId: r.member_id,
    username: r.username,
    isTranslator: r.is_translator,
    isProofreader: r.is_proofreader,
    isTypesetter: r.is_typesetter,
    isRedrawer: r.is_redrawer,
    updatedAt: r.updated_at,
  }));
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
  url: string;
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
        url: string;
      }[]
    >('get_project_files', {
      payload,
    });

    console.debug('[ipc] get_project_files result', { projectId, targetId, raw });

    return (raw || []).map(f => ({
      id: f.id,
      name: f.name,
      sourceCount: f.source_count ?? 0,
      url: f.url,
    }));
  } catch (err) {
    console.error('[ipc] getProjectFiles failed', { projectId, targetId, err });
    throw err;
  }
}

// ========== 获取页面的 sources（用于 TranslatorView） ==========

export interface PageTranslation {
  id: string;
  content: string;
  proofreadContent?: string;
  selected: boolean;
}

export interface PageSource {
  id: string;
  x: number;
  y: number;
  positionType: number;
  myTranslation?: PageTranslation;
  translations: PageTranslation[];
}

export async function getPageSources(fileId: string, targetId: string): Promise<PageSource[]> {
  try {
    console.debug('[ipc] invoke get_page_sources', { fileId, targetId });
    const raw = await invoke<
      {
        id: string;
        x: number;
        y: number;
        position_type: number;
        my_translation?: {
          id: string;
          content: string;
          proofread_content?: string;
          selected: boolean;
        };
        translations: {
          id: string;
          content: string;
          proofread_content?: string;
          selected: boolean;
        }[];
      }[]
    >('get_page_sources', {
      payload: {
        file_id: fileId,
        target_id: targetId,
      },
    });

    console.debug('[ipc] get_page_sources result', { fileId, targetId, raw });

    return (raw || []).map(s => ({
      id: s.id,
      x: s.x,
      y: s.y,
      positionType: s.position_type,
      myTranslation: s.my_translation
        ? {
            id: s.my_translation.id,
            content: s.my_translation.content,
            proofreadContent: s.my_translation.proofread_content,
            selected: s.my_translation.selected,
          }
        : undefined,
      translations: (s.translations || []).map(t => ({
        id: t.id,
        content: t.content,
        proofreadContent: t.proofread_content,
        selected: t.selected,
      })),
    }));
  } catch (err) {
    console.error('[ipc] getPageSources failed', { fileId, targetId, err });
    throw err;
  }
}

export interface CreateSourcePayload {
  fileId: string;
  targetId: string;
  positionType: number;
  x: number;
  y: number;
}

export async function createSource(payload: CreateSourcePayload): Promise<PageSource> {
  try {
    console.debug('[ipc] invoke create_source', payload);

    const raw = await invoke<{
      id: string;
      x: number;
      y: number;
      position_type: number;
      my_translation?: {
        id: string;
        content: string;
        proofread_content?: string | null;
        selected: boolean;
      };
      translations?: {
        id: string;
        content: string;
        proofread_content?: string | null;
        selected: boolean;
      }[];
    }>('create_source', {
      payload: {
        file_id: payload.fileId,
        target_id: payload.targetId,
        position_type: payload.positionType,
        x: payload.x,
        y: payload.y,
      },
    });

    console.debug('[ipc] create_source result', { payload, raw });

    return {
      id: raw.id,
      x: raw.x,
      y: raw.y,
      positionType: raw.position_type,
      myTranslation: raw.my_translation
        ? {
            id: raw.my_translation.id,
            content: raw.my_translation.content,
            proofreadContent:
              typeof raw.my_translation.proofread_content === 'string'
                ? raw.my_translation.proofread_content
                : undefined,
            selected: raw.my_translation.selected,
          }
        : undefined,
      translations: (raw.translations || []).map(item => ({
        id: item.id,
        content: item.content,
        proofreadContent:
          typeof item.proofread_content === 'string' ? item.proofread_content : undefined,
        selected: item.selected,
      })),
    };
  } catch (err) {
    console.error('[ipc] createSource failed', { payload, err });
    throw err;
  }
}

export interface UpdateSourcePayload {
  sourceId: string;
  positionType: number;
}

export async function updateSource(payload: UpdateSourcePayload): Promise<PageSource> {
  try {
    console.debug('[ipc] invoke update_source', payload);

    const raw = await invoke<{
      id: string;
      x: number;
      y: number;
      position_type: number;
      my_translation?: {
        id: string;
        content: string;
        proofread_content?: string | null;
        selected: boolean;
      };
      translations?: {
        id: string;
        content: string;
        proofread_content?: string | null;
        selected: boolean;
      }[];
    }>('update_source', {
      payload: {
        source_id: payload.sourceId,
        position_type: payload.positionType,
      },
    });

    console.debug('[ipc] update_source result', { payload, raw });

    return {
      id: raw.id,
      x: raw.x,
      y: raw.y,
      positionType: raw.position_type,
      myTranslation: raw.my_translation
        ? {
            id: raw.my_translation.id,
            content: raw.my_translation.content,
            proofreadContent:
              typeof raw.my_translation.proofread_content === 'string'
                ? raw.my_translation.proofread_content
                : undefined,
            selected: raw.my_translation.selected,
          }
        : undefined,
      translations: (raw.translations || []).map(item => ({
        id: item.id,
        content: item.content,
        proofreadContent:
          typeof item.proofread_content === 'string' ? item.proofread_content : undefined,
        selected: item.selected,
      })),
    };
  } catch (err) {
    console.error('[ipc] updateSource failed', { payload, err });
    throw err;
  }
}

export async function deleteSource(sourceId: string): Promise<void> {
  try {
    console.debug('[ipc] invoke delete_source', { sourceId });

    await invoke('delete_source', {
      payload: {
        source_id: sourceId,
      },
    });

    console.debug('[ipc] delete_source ok', { sourceId });
  } catch (err) {
    console.error('[ipc] deleteSource failed', { sourceId, err });
    throw err;
  }
}

export interface UpdateTranslationPayload {
  translationId: string;
  selected?: boolean;
  proofreadContent?: string;
  content?: string;
}

export async function submitTranslation(
  payload: SubmitTranslationPayload
): Promise<PageTranslation> {
  try {
    console.debug('[ipc] invoke submit_translation', payload);

    const raw = await invoke<{
      id: string;
      content: string;
      proofread_content?: string | null;
      selected: boolean;
    }>('submit_translation', {
      payload: {
        source_id: payload.sourceId,
        target_id: payload.targetId,
        content: payload.content,
      },
    });

    console.debug('[ipc] submit_translation result', { payload, raw });

    return {
      id: raw.id,
      content: raw.content,
      proofreadContent:
        typeof raw.proofread_content === 'string' ? raw.proofread_content : undefined,
      selected: raw.selected,
    };
  } catch (err) {
    console.error('[ipc] submitTranslation failed', { payload, err });
    throw err;
  }
}

export interface SubmitTranslationPayload {
  sourceId: string;
  targetId: string;
  content: string;
}

export interface UpdateTranslationPayload {
  translationId: string;
  selected?: boolean;
  proofreadContent?: string;
  content?: string;
}

export async function updateTranslation(
  payload: UpdateTranslationPayload
): Promise<PageTranslation> {
  const request: Record<string, unknown> = {
    translation_id: payload.translationId,
  };

  if (typeof payload.selected === 'boolean') {
    request.selected = payload.selected;
  }

  if (typeof payload.proofreadContent !== 'undefined') {
    request.proofread_content = payload.proofreadContent;
  }

  if (typeof payload.content !== 'undefined') {
    request.content = payload.content;
  }

  if (Object.keys(request).length === 1) {
    throw new Error('updateTranslation requires at least one field');
  }

  try {
    console.debug('[ipc] invoke update_translation', request);

    const raw = await invoke<{
      id: string;
      content: string;
      proofread_content?: string | null;
      selected: boolean;
    }>('update_translation', {
      payload: request,
    });

    console.debug('[ipc] update_translation result', { request, raw });

    return {
      id: raw.id,
      content: raw.content,
      proofreadContent:
        typeof raw.proofread_content === 'string' ? raw.proofread_content : undefined,
      selected: raw.selected,
    };
  } catch (err) {
    console.error('[ipc] updateTranslation failed', { request, err });
    throw err;
  }
}

export async function proxyImage(url: string): Promise<{ b64: string; content_type: string }> {
  try {
    console.debug('[ipc] invoke proxy_image', { url });
    const raw = await invoke<{ b64: string; content_type: string }>('proxy_image', {
      url,
    });

    console.debug('[ipc] proxy_image result', { url, raw });

    return raw as { b64: string; content_type: string };
  } catch (err) {
    console.error('[ipc] proxyImage failed', { url, err });
    throw err;
  }
}

// 上传项目文件（漫画页）
export async function uploadProjectFile(
  projectId: string,
  fileName: string,
  fileBytes: Uint8Array
): Promise<void> {
  try {
    console.debug('[ipc] invoke upload_project_file', {
      projectId,
      fileName,
      size: fileBytes.length,
    });

    // 将 Uint8Array 转换为 number[] 以符合 Tauri invoke 序列化
    const bytesArray = Array.from(fileBytes);

    await invoke<void>('upload_project_file', {
      payload: {
        project_id: projectId,
        file_name: fileName,
        file_bytes: bytesArray,
      },
    });

    console.debug('[ipc] upload_project_file success', { projectId, fileName });
  } catch (err) {
    console.error('[ipc] uploadProjectFile failed', { projectId, fileName, err });
    throw err;
  }
}
