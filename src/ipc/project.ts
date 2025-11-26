import { invoke } from '@tauri-apps/api/core';
import type { ResProjectSet, ResProject, ResProjectEnriched } from '../api/model/project';

// 获取指定汉化组的项目集列表
export async function getTeamProjectSets(params: {
  teamId: string;
  page: number;
  limit: number;
}): Promise<ResProjectSet[]> {
  // Tauri maps snake_case Rust args to camelCase in JS; send camelCase keys
  return await invoke<ResProjectSet[]>('get_team_project_sets', {
    teamId: params.teamId,
    page: params.page,
    limit: params.limit,
  });
}

// 获取指定汉化组下某项目集的项目列表
export async function getTeamProjects(params: {
  teamId: string;
  project_set: string;
  page: number;
  limit: number;
}): Promise<ResProject[]> {
  return await invoke<ResProject[]>('get_team_projects', {
    teamId: params.teamId,
    project_set: params.project_set,
    page: params.page,
    limit: params.limit,
  });
}

// 获取当前用户的项目列表
export async function getUserProjects(params: {
  page: number;
  limit: number;
}): Promise<ResProject[]> {
  return await invoke<ResProject[]>('get_user_projects', params);
}

// 获取当前用户的 enriched 项目列表
export async function getUserProjectsEnriched(params: {
  page: number;
  limit: number;
}): Promise<ResProjectEnriched[]> {
  return await invoke<ResProjectEnriched[]>('get_user_projects_enriched', params);
}

// PopRaKo 主导的项目搜索参数（与 PoprakoProjFilterReq 对应的前端版本）
export interface ProjectSearchFilters {
  fuzzy_proj_name?: string;
  translating_status?: number;
  proofreading_status?: number;
  typesetting_status?: number;
  reviewing_status?: number;
  is_published?: boolean;
  member_ids?: string[];
  time_start?: number;
  page?: number;
  limit?: number;
  [key: string]: unknown;
}

// 基于 PopRaKo /projs/search + Moetran /user/projects?word= 的用户项目搜索
export async function searchUserProjectsEnriched(
  filters: ProjectSearchFilters
): Promise<ResProjectEnriched[]> {
  return await invoke<ResProjectEnriched[]>('search_user_projects_enriched', filters);
}

// 基于 PopRaKo /projs/search + Moetran /teams/:team_id/projects?word= 的团队项目搜索
export async function searchTeamProjectsEnriched(
  params: {
    teamId: string;
  } & ProjectSearchFilters
): Promise<ResProjectEnriched[]> {
  // send camelCase keys to match Tauri's expected arg names
  const { teamId, ...rest } = params as any;
  return await invoke<ResProjectEnriched[]>('search_team_projects_enriched', {
    teamId,
    ...rest,
  });
}

// 获取团队的 enriched 项目列表（无筛选，分页）
export async function getTeamProjectsEnriched(params: {
  teamId: string;
  page: number;
  limit: number;
}): Promise<ResProjectEnriched[]> {
  return await invoke<ResProjectEnriched[]>('get_team_projects_enriched', {
    teamId: params.teamId,
    page: params.page,
    limit: params.limit,
  });
}

// PopRaKo 创建项目集请求参数
export interface CreateProjsetPayload {
  projset_name: string;
  projset_description: string;
  team_id: string;
  mtr_token: string;
}

export interface CreateProjsetResult {
  projset_serial: number;
}

// 调用 PopRaKo /projset/create 的 IPC 封装
export async function createProjset(payload: CreateProjsetPayload): Promise<CreateProjsetResult> {
  const { projset_name, projset_description, team_id, mtr_token } = payload;

  return await invoke<CreateProjsetResult>('create_projset', {
    projset_name,
    projset_description,
    team_id,
    mtr_token,
  });
}

// PopRaKo 创建项目请求参数
export interface CreateProjPayload {
  proj_name: string;
  proj_description: string;
  team_id: string;
  projset_id: string;
  mtr_auth: string;
  workset_index: number;
  source_language: string;
  target_languages: string[];
  allow_apply_type: number;
  application_check_type: number;
  default_role: string;
}

export interface CreateProjResult {
  proj_id: string;
  proj_serial: number;
  projset_index: number;
}

// 调用 PopRaKo /proj/create 的 IPC 封装
export async function createProj(payload: CreateProjPayload): Promise<CreateProjResult> {
  return await invoke<CreateProjResult>('create_proj', payload as any);
}

// PopRaKo 项目集 DTO（简化版，只保留 Creator 需要的字段）
export interface PoprakoProjsetInfo {
  projset_id: string;
  projset_name: string;
  projset_description?: string | null;
  projset_serial: number;
  team_id: string;
}

// 获取 PopRaKo 中指定团队下的项目集列表
export async function getTeamPoprakoProjsets(teamId: string): Promise<PoprakoProjsetInfo[]> {
  return await invoke<PoprakoProjsetInfo[]>('get_team_poprako_projsets', { teamId });
}

// 指派成员到 PopRaKo 项目
export interface AssignMemberPayload {
  proj_id: string;
  member_id: string;
  is_translator: boolean;
  is_proofreader: boolean;
  is_typesetter: boolean;
  is_principal: boolean;
}

export async function assignMemberToProj(payload: AssignMemberPayload): Promise<void> {
  await invoke<void>('assign_member_to_proj', payload as any);
}
