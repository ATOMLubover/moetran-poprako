import { ResTeam } from './team';

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
}
