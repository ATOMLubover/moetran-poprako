<script setup lang="ts">
import { ref } from 'vue';
import TranslatorView from './views/translator/TranslatorView.vue';

const projectId = ref('demo-project');

const pageIndex = ref(0);

const showTranslator = ref(true);

function handleBack(): void {
  showTranslator.value = false;
}

function handleOpenTranslator(): void {
  showTranslator.value = true;
}
</script>

<template>
  <main class="app-root">
    <TranslatorView
      v-if="showTranslator"
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
