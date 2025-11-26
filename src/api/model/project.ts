import { ResTeam } from './team';

export interface ResProjectSet {
  id: string;
  name: string;
}

export interface ResProject {
  id: string;
  name: string;

  source_count: number;
  translated_source_count: number;
  checked_source_count: number;

  team: ResTeam;
  project_set: ResProjectSet;
}

// enriched 项目 DTO（Moetran + PopRaKo）
export interface ResProjectEnriched extends ResProject {
  has_poprako: boolean;
  projset_index?: number;
  projset_serial?: number;
  translating_status?: number;
  proofreading_status?: number;
  typesetting_status?: number;
  reviewing_status?: number;
  is_published?: boolean;
}
