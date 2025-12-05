<script setup lang="ts">
import { ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useToastStore } from '../stores/toast';
import { useTokenStore } from '../stores/token';
import { createPoprakoProjset } from '../ipc/project';

// Props: 从父组件注入当前选中的团队 ID
const props = defineProps<{ teamId?: string | null }>();

const emit = defineEmits<{ (e: 'close'): void; (e: 'created'): void }>();

// 表单数据
const projsetName = ref<string>('');
const projsetDescription = ref<string>('');

// 提交状态
const loading = ref<boolean>(false);

// Stores
const toastStore = useToastStore();
const tokenStore = useTokenStore();
const { moetranToken } = storeToRefs(tokenStore);

// 关闭视图
function handleClose(): void {
  emit('close');
}

// 提交表单：创建项目集
async function handleCreateProjset(): Promise<void> {
  if (!projsetName.value) {
    toastStore.show('请填写项目集名称');
    return;
  }

  if (!props.teamId) {
    toastStore.show('请先在左侧选择一个汉化组');
    return;
  }

  if (!moetranToken.value) {
    toastStore.show('缺少 Moetran Token，请先登录');
    return;
  }

  loading.value = true;

  try {
    await createPoprakoProjset({
      projsetName: projsetName.value,
      projsetDescription: projsetDescription.value,
      teamId: props.teamId,
      mtrToken: moetranToken.value,
    });

    toastStore.show('项目集创建成功');
    emit('created');
    emit('close');
  } catch (err) {
    console.error('Create projset failed', err);
    toastStore.show(`项目集创建失败：${String(err)}`);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <section class="projset-creator-root">
    <header class="projset-creator-header">
      <div class="projset-creator-header__left">
        <h1 class="projset-creator-title">创建新项目集</h1>
      </div>
      <div class="projset-creator-header__right">
        <button
          type="button"
          class="projset-creator-close"
          @click="handleClose"
          :disabled="loading"
        >
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </header>

    <div class="projset-creator-body">
      <div class="projset-creator-form">
        <!-- 项目集名称 -->
        <div class="form-field">
          <label for="projset-name" class="form-label">项目集名称*</label>
          <input
            id="projset-name"
            v-model="projsetName"
            type="text"
            class="form-input"
            placeholder="如：主线文本、活动文本等"
            :disabled="loading"
          />
        </div>

        <!-- 项目集描述 -->
        <div class="form-field">
          <label for="projset-desc" class="form-label">项目集描述</label>
          <textarea
            id="projset-desc"
            v-model="projsetDescription"
            class="form-textarea"
            rows="4"
            placeholder="项目集用途说明（可选）"
            :disabled="loading"
          ></textarea>
        </div>
      </div>
    </div>

    <footer class="projset-creator-footer">
      <button type="button" class="btn-cancel" @click="handleClose" :disabled="loading">
        取消
      </button>
      <button
        type="button"
        class="btn-submit"
        @click="handleCreateProjset"
        :disabled="loading || !projsetName"
      >
        {{ loading ? '创建中...' : '创建项目集' }}
      </button>
    </footer>
  </section>
</template>

<style scoped>
.projset-creator-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: linear-gradient(135deg, #f0f4f8 0%, #fafbfd 100%);
  overflow: hidden;
}

.projset-creator-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 28px;
  border-bottom: 1px solid rgba(210, 220, 235, 0.5);
  background: rgba(255, 255, 255, 0.95);
}

.projset-creator-header__left {
  flex: 1;
}

.projset-creator-title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  color: #1f2e43;
  letter-spacing: 0.5px;
}

.projset-creator-close {
  border: none;
  background: none;
  cursor: pointer;
  padding: 6px;
  color: #6d7a8a;
  transition: color 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.projset-creator-close:hover {
  color: #2f5a8f;
}

.projset-creator-close:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.projset-creator-body {
  flex: 1;
  padding: 24px 28px;
  overflow-y: auto;
}

.projset-creator-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 600px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  font-size: 14px;
  font-weight: 600;
  color: #294061;
}

.form-input,
.form-textarea {
  padding: 10px 14px;
  border: 1px solid rgba(210, 220, 235, 0.65);
  border-radius: 8px;
  font-size: 14px;
  color: #28405c;
  background: #fff;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: #76b8ff;
  box-shadow: 0 0 0 3px rgba(118, 184, 255, 0.1);
}

.form-input:disabled,
.form-textarea:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background: #f6f8fa;
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.projset-creator-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 28px;
  border-top: 1px solid rgba(210, 220, 235, 0.5);
  background: rgba(255, 255, 255, 0.95);
}

.btn-cancel,
.btn-submit {
  padding: 10px 20px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease,
    background 0.15s ease;
}

.btn-cancel {
  border: 1px solid rgba(200, 208, 218, 0.6);
  background: #f6f8fa;
  color: #6d7a8a;
}

.btn-cancel:hover:not(:disabled) {
  background: #eef1f5;
  transform: translateY(-1px);
}

.btn-submit {
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: linear-gradient(135deg, #5ba3e0, #6db4f0);
  color: #fff;
  box-shadow: 0 6px 16px rgba(118, 184, 255, 0.2);
}

.btn-submit:hover:not(:disabled) {
  background: linear-gradient(135deg, #4a92cf, #5ca3df);
  transform: translateY(-2px);
  box-shadow: 0 10px 24px rgba(118, 184, 255, 0.35);
}

.btn-cancel:disabled,
.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}
</style>
