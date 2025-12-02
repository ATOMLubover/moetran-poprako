import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ViewName = 'login' | 'panel' | 'projectDetail' | 'translator';

export interface TranslatorViewParams {
  projectId: string;
  targetId: string;
  files: Array<{ id: string; name: string; sourceCount: number; url: string }>;
  enabled: boolean;
  initialMode: 'translate' | 'read';
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
  role?: any | null;
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
