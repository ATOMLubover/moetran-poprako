import { invoke } from '@tauri-apps/api/core';
import type { ResProjectSet, ResProject, ResProjectEnriched } from '../api/model/project';

// 获取指定汉化组的项目集列表
export async function getTeamProjectSets(params: {
  team_id: string;
  page: number;
  limit: number;
}): Promise<ResProjectSet[]> {
  return await invoke<ResProjectSet[]>('get_team_project_sets', params);
}

// 获取指定汉化组下某项目集的项目列表
export async function getTeamProjects(params: {
  team_id: string;
  project_set: string;
  page: number;
  limit: number;
}): Promise<ResProject[]> {
  return await invoke<ResProject[]>('get_team_projects', params);
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
