<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, ref } from 'vue';
import { getCaptcha as fetchCaptchaApi, aquireMoetranToken } from '../api/moetran';
import type { ReqMoeToken } from '../api/model/auth';
import AppToast from '../components/AppToast.vue';

type ToastTone = 'success' | 'error';

interface ToastState {
  message: string;
  tone: ToastTone;
  visible: boolean;
}

// 邮箱输入
const email = ref('');

// 密码输入
const password = ref('');

// 验证码输入
const captcha = ref('');

// 验证码图片 Base64
const captchaImage = ref('');

// 验证码附带信息
const captchaInfo = ref('');

// 验证码容器与尺寸
const captchaWrapperRef = ref<HTMLDivElement | null>(null);
const captchaNatural = reactive({ width: 0, height: 0 });
const captchaWrapperHeight = ref<number | null>(null);

function updateCaptchaWrapperHeight(): void {
  const wrapper = captchaWrapperRef.value;
  if (!wrapper || captchaNatural.width <= 0 || captchaNatural.height <= 0) return;

  const wrapperWidth = wrapper.clientWidth;
  if (wrapperWidth <= 0) return;

  const scale = wrapperWidth / captchaNatural.width;
  const nextHeight = Math.round(captchaNatural.height * scale);
  captchaWrapperHeight.value = nextHeight;
}

function handleCaptchaLoad(event: Event): void {
  const img = event.target as HTMLImageElement | null;
  if (!img) return;
  captchaNatural.width = img.naturalWidth;
  captchaNatural.height = img.naturalHeight;
  updateCaptchaWrapperHeight();
}

// 加载状态
const isLoading = ref(false);

const toastState = reactive<ToastState>({
  message: '',
  tone: 'success',
  visible: false,
});

let toastTimer: number | null = null;

function showToast(message: string, tone: ToastTone = 'success', duration = 2400): void {
  toastState.message = message;
  toastState.tone = tone;
  toastState.visible = true;

  if (toastTimer !== null) {
    window.clearTimeout(toastTimer);
  }

  toastTimer = window.setTimeout(() => {
    toastState.visible = false;
    toastTimer = null;
  }, duration);
}

// 获取验证码
async function fetchCaptcha(): Promise<void> {
  try {
    const data = await fetchCaptchaApi();

    captchaImage.value = data.image;

    captchaInfo.value = data.info;

    captcha.value = '';

    // 清空旧高度，等待新图 onload 后计算
    captchaWrapperHeight.value = null;
  } catch (error) {
    console.error('获取验证码时出现异常', error);

    const message =
      error instanceof Error && error.message.includes('status')
        ? '验证码加载失败，请稍后再试'
        : '验证码加载失败，请检查网络';

    showToast(message, 'error');
  }
}

// 处理登录逻辑
async function handleLogin(): Promise<void> {
  if (!email.value || !password.value || !captcha.value) {
    console.warn('请填写完整信息');

    showToast('请填写完整信息', 'error');

    return;
  }

  isLoading.value = true;

  try {
    const payload: ReqMoeToken = {
      email: email.value,
      password: password.value,
      captcha: captcha.value,
      info: captchaInfo.value,
    };

    const tokenResponse = await aquireMoetranToken(payload);

    console.log('登录成功', tokenResponse.token);

    showToast('登录成功', 'success');
  } catch (error) {
    console.error('登录失败', error);

    await fetchCaptcha();

    const message =
      error instanceof Error && error.message.includes('status')
        ? '登录失败，请检查输入后重试'
        : '登录失败，请稍后再试';

    showToast(message, 'error');
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  fetchCaptcha();
  window.addEventListener('resize', updateCaptchaWrapperHeight);
});

onBeforeUnmount(() => {
  if (toastTimer !== null) {
    window.clearTimeout(toastTimer);

    toastTimer = null;
  }
  window.removeEventListener('resize', updateCaptchaWrapperHeight);
});
</script>

<template>
  <div class="login-view">
    <div class="login-card">
      <div class="login-card__header">
        <h1 class="login-card__title">欢迎回来</h1>
        <p class="login-card__subtitle">登录到 Moetran Native</p>
      </div>

      <form class="login-form" @submit.prevent="handleLogin">
        <div class="form-group">
          <label class="form-label">邮箱</label>
          <input
            v-model="email"
            type="email"
            class="form-input"
            placeholder="请输入邮箱"
            required
          />
        </div>

        <div class="form-group">
          <label class="form-label">密码</label>
          <input
            v-model="password"
            type="password"
            class="form-input"
            placeholder="请输入密码"
            required
          />
        </div>

        <div class="form-group">
          <label class="form-label">验证码</label>
          <div class="captcha-row">
            <input
              v-model="captcha"
              type="text"
              class="form-input captcha-input"
              placeholder="验证码"
              required
            />
            <div
              class="captcha-image-wrapper"
              ref="captchaWrapperRef"
              @click="fetchCaptcha"
              title="点击刷新验证码"
              :style="{ height: captchaWrapperHeight ? captchaWrapperHeight + 'px' : undefined }"
            >
              <img
                v-if="captchaImage"
                :src="captchaImage"
                alt="验证码"
                class="captcha-image"
                @load="handleCaptchaLoad"
              />
              <span v-else class="captcha-placeholder">加载中...</span>
            </div>
          </div>
        </div>

        <button type="submit" class="login-button" :disabled="isLoading">
          <span v-if="isLoading">登录中...</span>
          <span v-else>登录</span>
        </button>
      </form>
    </div>
    <AppToast :visible="toastState.visible" :message="toastState.message" :tone="toastState.tone" />
  </div>
</template>

<style scoped>
.login-view {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: #f4f9ff;
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #28384c;
}

.login-card {
  width: 100%;
  max-width: 380px;
  padding: 36px;
  border-radius: 20px;
  background: #ffffff;
  box-shadow: 0 12px 32px rgba(118, 166, 219, 0.12);
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.login-card__header {
  text-align: center;
}

.login-card__title {
  margin: 0 0 6px;
  font-size: 24px;
  font-weight: 600;
  color: #1f2e43;
}

.login-card__subtitle {
  margin: 0;
  font-size: 13px;
  color: #5a6c86;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  font-weight: 600;
  color: #4a5f7d;
  margin-left: 2px;
}

.form-input {
  width: 100%;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid rgba(188, 206, 233, 0.6);
  background: #f8fbff;
  color: #2a3b4f;
  font-size: 14px;
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease;
  box-sizing: border-box;
}

.form-input:focus {
  border-color: #70b6ff;
  box-shadow: 0 0 0 3px rgba(112, 182, 255, 0.15);
}

.form-input::placeholder {
  color: #aab8cc;
}

.captcha-row {
  display: flex;
  gap: 10px;
}

.captcha-input {
  flex: 1;
}

.captcha-image-wrapper {
  flex: 0 0 140px;
  /* Allow image to define height while keeping a reasonable min height */
  min-height: 54px;
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid rgba(188, 206, 233, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  background: #ffffff;
  box-shadow: 0 2px 4px rgba(118, 166, 219, 0.18);
}

.captcha-image {
  display: block;
  width: 100%;
  height: auto;
  object-fit: contain; /* ensure full image visible */
}

.captcha-placeholder {
  font-size: 12px;
  color: #a0b0c8;
}

.login-button {
  margin-top: 8px;
  padding: 12px;
  border: none;
  border-radius: 10px;
  background: #9ecaf3;
  color: #ffffff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    background-color 0.16s ease;
}

.login-button:hover:not(:disabled) {
  background: #5480bd;
  transform: translateY(-1px);
}

.login-button:active:not(:disabled) {
  transform: translateY(0);
}

.login-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
