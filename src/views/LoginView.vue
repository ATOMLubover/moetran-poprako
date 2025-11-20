<script setup lang="ts">
import { onMounted, ref } from 'vue';

// 邮箱输入
const email = ref('');

// 密码输入
const password = ref('');

// 验证码输入
const captcha = ref('');

// 验证码图片 Base64
const captchaImage = ref('');

// 加载状态
const isLoading = ref(false);

// 模拟获取验证码
// TODO: 对接真实后端接口
async function __mockFetchCaptcha() {
  // 这里使用一个简单的 mock base64 图片（一个 1x1 的透明像素或者简单的色块，实际开发中替换为真实数据）
  // 为了演示效果，这里放一个简单的 SVG 转 Base64
  const svg = `
  <svg width="100" height="40" xmlns="http://www.w3.org/2000/svg">
    <rect width="100%" height="100%" fill="#f0f8ff"/>
    <text x="50%" y="50%" font-family="Arial" font-size="20" fill="#3b82f6" dominant-baseline="middle" text-anchor="middle">1234</text>
  </svg>`;

  captchaImage.value = 'data:image/svg+xml;base64,' + btoa(svg);
}

// 处理登录逻辑
// TODO: 对接真实后端接口
async function __mockHandleLogin() {
  if (!email.value || !password.value || !captcha.value) {
    console.warn('请填写完整信息');
    return;
  }

  isLoading.value = true;

  console.log('正在登录...', {
    email: email.value,
    password: password.value,
    captcha: captcha.value,
  });

  try {
    // 模拟网络请求延迟
    await new Promise(resolve => setTimeout(resolve, 1500));

    console.log('登录成功');

    // TODO: 跳转到主页或项目列表页
  } catch (error) {
    console.error('登录失败', error);
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  __mockFetchCaptcha();
});
</script>

<template>
  <div class="login-view">
    <div class="login-card">
      <div class="login-card__header">
        <h1 class="login-card__title">欢迎回来</h1>
        <p class="login-card__subtitle">登录到 Moetran Native</p>
      </div>

      <form class="login-form" @submit.prevent="__mockHandleLogin">
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
            <div class="captcha-image-wrapper" @click="__mockFetchCaptcha" title="点击刷新验证码">
              <img v-if="captchaImage" :src="captchaImage" alt="验证码" class="captcha-image" />
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
  flex: 0 0 90px;
  height: 42px;
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid rgba(188, 206, 233, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fff;
}

.captcha-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
