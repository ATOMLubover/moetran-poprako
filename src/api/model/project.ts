import { ResTeam } from './team';

export interface ResProject {
  id: string;
  name: string;

  source_count: number;
  translated_source_count: number;
  checked_source_count: number;

  team: ResTeam;

  project_set: ResProjectSet;
}

export interface ResProjectSet {
  id: string;
  name: string;
}
