<script setup lang="ts">
import { ref, watch } from 'vue';
import TranslatorView from './views/TranslatorView.vue';
import LoginView from './views/LoginView.vue';
import PanelView from './views/PanelView.vue';
import AppToast from './components/AppToast.vue';
import { useToastStore } from './stores/toast';
import { useRouterStore } from './stores/router';
import ProjectDetailView from './views/ProjectDetailView.vue';

const toastStore = useToastStore();
const routerStore = useRouterStore();

const pageIndex = ref(0);

// Reset pageIndex whenever entering translator view
watch(
  () => routerStore.currentView,
  v => {
    if (v === 'translator') {
      pageIndex.value = 0;
    }
  }
);
</script>

<template>
  <main class="app-root">
    <LoginView v-if="routerStore.currentView === 'login'" @logged="routerStore.navigateToPanel()" />
    <PanelView v-else-if="routerStore.currentView === 'panel'" />
    <ProjectDetailView
      v-else-if="routerStore.currentView === 'projectDetail' && routerStore.projectDetailParams"
      :project-id="routerStore.projectDetailParams.projectId"
      :title="routerStore.projectDetailParams.title"
      :projset-name="routerStore.projectDetailParams.projsetName"
      :projset-index="routerStore.projectDetailParams.projsetIndex"
      :total-markers="routerStore.projectDetailParams.totalMarkers"
      :total-translated="routerStore.projectDetailParams.totalTranslated"
      :total-checked="routerStore.projectDetailParams.totalChecked"
      :translating-status="routerStore.projectDetailParams.translatingStatus"
      :proofreading-status="routerStore.projectDetailParams.proofreadingStatus"
      :typesetting-status="null"
      :reviewing-status="null"
      :translators="[]"
      :proofreaders="[]"
      :letterers="[]"
      :reviewers="[]"
      :role="routerStore.projectDetailParams.role"
      :has-poprako="routerStore.projectDetailParams.hasPoprako ?? undefined"
      @close="routerStore.navigateToPanel()"
    />
    <TranslatorView
      v-else-if="routerStore.currentView === 'translator' && routerStore.translatorParams"
      :project-id="routerStore.translatorParams.projectId"
      :target-id="routerStore.translatorParams.targetId"
      :files="routerStore.translatorParams.files"
      :is-enabled="routerStore.translatorParams.enabled"
      :initial-mode="routerStore.translatorParams.initialMode"
      :has-poprako="routerStore.translatorParams.hasPoprako"
      :is-proofreader="routerStore.translatorParams.isProofreader"
      v-model:page-index="pageIndex"
      @back="routerStore.navigateToPanel()"
    />

    <!-- 全局 Toast -->
    <AppToast :visible="toastStore.visible" :message="toastStore.message" :tone="toastStore.tone" />
  </main>
</template>

<style scoped>
:global(body) {
  margin: 0;
  background: #f4f8ff;
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #2a3b4f;
}

.app-root {
  min-height: 100vh;
}

.placeholder p {
  margin: 0;
  max-width: 480px;
  line-height: 1.6;
  color: #4a5f7a;
}

.placeholder__action {
  padding: 10px 22px;
  border-radius: 999px;
  border: 1px solid rgba(118, 184, 255, 0.7);
  background: rgba(241, 247, 255, 0.95);
  color: #2f5a8f;
  font-size: 14px;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease;
}

.placeholder__action:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 24px rgba(136, 190, 247, 0.28);
}
.placeholder__actions {
  display: flex;
  flex-direction: row;
  gap: 16px;
  margin-top: 8px;
}
</style>
