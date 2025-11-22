<script setup lang="ts">
import { ref } from 'vue';
import TranslatorView from './views/TranslatorView.vue';
import LoginView from './views/LoginView.vue';
import PanelView from './views/PanelView.vue';
import AppToast from './components/AppToast.vue';
import { useToastStore } from './stores/toast';

type ViewName = 'login' | 'translator' | 'placeholder' | 'panel';

const projectId = ref('demo-project');

const pageIndex = ref(0);

const currentView = ref<ViewName>('login');
const toastStore = useToastStore();

function handleBack(): void {
  currentView.value = 'placeholder';
}

function handleOpenTranslator(): void {
  currentView.value = 'translator';
}
</script>

<template>
  <main class="app-root">
    <LoginView v-if="currentView === 'login'" @logged="currentView = 'panel'" />
    <PanelView v-else-if="currentView === 'panel'" />
    <TranslatorView
      v-else-if="currentView === 'translator'"
      :project-id="projectId"
      v-model:page-index="pageIndex"
      @back="handleBack"
    />
    <section v-else class="placeholder">
      <h1>项目详情（待实现）</h1>
      <p>这里将展示项目元信息、分工等内容。点击下方按钮可进入翻译工作台。</p>
      <button type="button" class="placeholder__action" @click="handleOpenTranslator">
        进入翻译工作台
      </button>
    </section>

    <!-- 全局 Toast -->
    <AppToast :visible="toastStore.visible" :message="toastStore.message" :tone="toastStore.tone" />

    <!-- 临时调试导航栏 -->
    <div class="debug-nav">
      <button @click="currentView = 'login'" :class="{ active: currentView === 'login' }">
        登录页
      </button>
      <button
        @click="currentView = 'placeholder'"
        :class="{ active: currentView === 'placeholder' }"
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
</style>
