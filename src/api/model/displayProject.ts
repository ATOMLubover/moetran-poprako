// 共享的项目展示类型定义（用于 ProjectList 与 PanelView 之间的数据契约）
// 所有注释使用中文，保持备忘录式说明

export type WorkPhase = 'translate' | 'proof' | 'typeset' | 'review' | 'publish';
export type PhaseStatus = 'unset' | 'pending' | 'wip' | 'completed';

export interface PhaseChip {
  phase: WorkPhase;
  status: PhaseStatus;
  label: string;
}

export interface ProjectBasicInfo {
  id: number;
  index: number;
  author: string;
  title: string;
  projectSetId?: number;
  phases: PhaseChip[];
}
