import type { ResMember } from './member';

// 展示视图阶段标识
export type WorkPhase = 'translate' | 'proof' | 'typeset' | 'redraw' | 'review' | 'publish';

// 展示视图阶段状态
export type PhaseStatus = 'unset' | 'pending' | 'wip' | 'completed';

// 阶段标签结构，包含成员文本
export interface PhaseChip {
  phase: WorkPhase;
  status: PhaseStatus;
  label: string;
  memberNames: string[];
}

// 纵览表格使用的项目信息
export interface ShownProjectInfo {
  id: string;
  title: string;
  description?: string | null;
  phases: PhaseChip[];
  translatedSourceCount: number;
  proofreadSourceCount: number;
  members: ResMember[];
  projsetSerial?: number | null;
  projsetIndex?: number | null;
  isPublished: boolean;
  translatingStatus: number | null;
  proofreadingStatus: number | null;
  typesettingStatus: number | null;
  redrawingStatus?: number | null;
  reviewingStatus: number | null;
  translatorNames: string[];
  proofreaderNames: string[];
  typesetterNames: string[];
  redrawerNames?: string[];
  reviewerNames: string[];
}
