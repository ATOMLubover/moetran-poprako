import { defineStore } from 'pinia';
import { ref } from 'vue';

// 私有类型：文件信息（用于 TranslatorViewParams.files）
interface _RouterFileInfo {
  id: string;
  name: string;
  sourceCount: number;
  url: string;
}

// 私有类型：项目详情传递中的 role（结构未知；前端仅检测是否为 null，避免使用 any）
interface _RouterProjectRole {
  [key: string]: unknown;
}

export type ViewName = 'login' | 'panel' | 'projectDetail' | 'translator';

export interface TranslatorViewParams {
  projectId: string;
  targetId: string;
  files: _RouterFileInfo[];
  enabled: boolean;
  initialMode: 'translate' | 'read';
  hasPoprako: boolean;
  isProofreader: boolean;
}

export interface ProjectDetailViewParams {
  projectId: string;
  title: string;
  projsetName: string | null;
  projsetIndex: number | null;
  totalMarkers: number;
  totalTranslated: number;
  totalChecked: number;
  translatingStatus: number | null;
  proofreadingStatus: number | null;
  // optional passthroughs for detail view (role for native Moetran projects)
  role?: _RouterProjectRole | null;
  hasPoprako?: boolean | null;
}

export const useRouterStore = defineStore('router', () => {
  const currentView = ref<ViewName>('login');

  // Translator 视图参数
  const translatorParams = ref<TranslatorViewParams | null>(null);

  // ProjectDetail 视图参数
  const projectDetailParams = ref<ProjectDetailViewParams | null>(null);

  function navigateToLogin() {
    currentView.value = 'login';
  }

  function navigateToPanel() {
    currentView.value = 'panel';
  }

  function navigateToProjectDetail(params: ProjectDetailViewParams) {
    projectDetailParams.value = params;
    currentView.value = 'projectDetail';
  }

  function navigateToTranslator(params: TranslatorViewParams) {
    translatorParams.value = params;
    currentView.value = 'translator';
  }

  return {
    currentView,
    translatorParams,
    projectDetailParams,
    navigateToLogin,
    navigateToPanel,
    navigateToProjectDetail,
    navigateToTranslator,
  };
});
