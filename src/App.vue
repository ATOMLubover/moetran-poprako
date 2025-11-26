<script setup lang="ts">
import { ref } from 'vue';
import TranslatorView from './views/TranslatorView.vue';
import LoginView from './views/LoginView.vue';
import PanelView from './views/PanelView.vue';
import AppToast from './components/AppToast.vue';
import { useToastStore } from './stores/toast';
import ProjectDetailView from './views/ProjectDetailView.vue';

// 视图名称枚举：新增 projectDetail
type ViewName = 'login' | 'translator' | 'projectDetail' | 'panel';

// 项目详情使用字符串 ID（已统一为 uuid/string）
const projectDetailId = ref('1');
const projectId = ref('demo-project');

const pageIndex = ref(0);

const currentView = ref<ViewName>('login');
// 翻译页面是否具备编辑权限（项目参与者）
const translatorEnabled = ref<boolean>(true);
const toastStore = useToastStore();

function handleBack(): void {
  // 返回到项目详情（便于来回测试）
  currentView.value = 'projectDetail';
}
</script>

<template>
  <main class="app-root">
    <LoginView v-if="currentView === 'login'" @logged="currentView = 'panel'" />
    <PanelView v-else-if="currentView === 'panel'" />
    <ProjectDetailView
      v-else-if="currentView === 'projectDetail'"
      :project-id="projectDetailId"
      @close="currentView = 'panel'"
      @open-translator="
        (enabled: boolean) => {
          translatorEnabled = enabled;
          currentView = 'translator';
        }
      "
    />
    <TranslatorView
      v-else-if="currentView === 'translator'"
      :project-id="projectId"
      :is-enabled="translatorEnabled"
      v-model:page-index="pageIndex"
      @back="handleBack"
    />

    <!-- 全局 Toast -->
    <AppToast :visible="toastStore.visible" :message="toastStore.message" :tone="toastStore.tone" />

    <!-- 临时调试导航栏 -->
    <div class="debug-nav">
      <button @click="currentView = 'login'" :class="{ active: currentView === 'login' }">
        登录页
      </button>
      <button
        @click="currentView = 'projectDetail'"
        :class="{ active: currentView === 'projectDetail' }"
      >
        项目详情
      </button>
      <button @click="currentView = 'translator'" :class="{ active: currentView === 'translator' }">
        翻译页
      </button>
      <button @click="currentView = 'panel'" :class="{ active: currentView === 'panel' }">
        面板
      </button>
    </div>
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
  padding-bottom: 60px; /* 为调试栏留出空间 */
}

.debug-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 50px;
  background: rgba(30, 40, 50, 0.9);
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 20px;
  z-index: 9999;
  backdrop-filter: blur(4px);
}

.debug-nav button {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: #fff;
  padding: 6px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.debug-nav button.active {
  background: rgba(100, 180, 255, 0.3);
  border-color: #64b4ff;
  color: #64b4ff;
}

.placeholder {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 40px;
  text-align: center;
  background: linear-gradient(180deg, #f6fbff 0%, #ffffff 80%);
}

.placeholder h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
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
