import { ResTeam } from './team';
import { ResMember } from './member';

// 私有类型：role passthrough，用于 Moetran 原生项目的 role 字段
interface _ProjectRole {
  [key: string]: unknown;
}

export interface ResProjectSet {
  id: string;
  name: string;
}

export interface ResProject {
  id: string;
  name: string;

  sourceCount: number;
  translatedSourceCount: number;
  checkedSourceCount: number;

  team: ResTeam;
  projectSet: ResProjectSet;
}

// enriched 项目 DTO（Moetran + PopRaKo）
export interface ResProjectEnriched extends ResProject {
  hasPoprako: boolean;
  projsetIndex?: number;
  projsetSerial?: number;
  translatingStatus?: number;
  proofreadingStatus?: number;
  typesettingStatus?: number;
  reviewingStatus?: number;
  isPublished?: boolean;
  // PopRaKo 返回的成员信息（可选）
  members?: ResMember[];
  // 仅包含 memberId 的负责人列表（可选）
  // 仅包含 userId 的负责人列表（可选，用于与当前用户 id 比对）
  principals?: string[];
  // Moetran 原生项目返回的 role 字段（若用户在项目内则为对象，否则为 null）
  // 只需用于判定是否为项目成员，不依赖具体结构
  role?: _ProjectRole | null;
}
