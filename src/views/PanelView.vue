<script setup lang="ts">
import { onMounted } from 'vue';
import { useTokenStore } from '../stores/token';

// 面板页面：展示当前两个 token 的摘要与操作按钮
const tokenStore = useTokenStore();

function handleLogoutAll(): void {
  // 前端清除并提示重新登录
  tokenStore.clearAll();
  window.location.reload();
}

onMounted(() => {
  // 可选：后续可以在这里拉取用户 profile
});
</script>

<template>
  <div class="panel-view">
    <h1 class="panel-title">控制面板</h1>
    <div class="token-block" :class="{ 'token-block--disabled': !tokenStore.moetranToken }">
      <h2 class="token-title">Moetran Token</h2>
      <p class="token-value" v-if="tokenStore.moetranToken">
        已加载：{{ tokenStore.moetranToken.slice(0, 16) }}...
      </p>
      <p class="token-empty" v-else>未登录</p>
    </div>
    <div class="token-block" :class="{ 'token-block--disabled': !tokenStore.poprakoToken }">
      <h2 class="token-title">Poprako Token</h2>
      <p class="token-value" v-if="tokenStore.poprakoToken">
        已加载：{{ tokenStore.poprakoToken.slice(0, 16) }}...
      </p>
      <p class="token-empty" v-else>未登录或未选择</p>
    </div>
    <button class="logout-btn" type="button" @click="handleLogoutAll">退出并清除缓存</button>
  </div>
</template>

<style scoped>
.panel-view {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  padding: 40px 28px;
  box-sizing: border-box;
  background: linear-gradient(180deg, #f4f9ff 0%, #ffffff 80%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
}
.panel-title {
  margin: 0 0 24px;
  font-size: 24px;
  font-weight: 600;
}
.token-block {
  width: 100%;
  max-width: 600px;
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 12px;
  padding: 18px 20px;
  margin-bottom: 20px;
  box-shadow: 0 8px 24px rgba(140, 180, 230, 0.15);
}
.token-block--disabled {
  opacity: 0.55;
  filter: grayscale(0.3);
}
.token-title {
  margin: 0 0 8px;
  font-size: 16px;
  font-weight: 600;
}
.token-value {
  margin: 0;
  font-size: 13px;
  color: #2a3b4f;
  word-break: break-all;
}
.token-empty {
  margin: 0;
  font-size: 13px;
  color: #8aa2bb;
}
.logout-btn {
  margin-top: 8px;
  padding: 10px 22px;
  border: 1px solid rgba(118, 184, 255, 0.7);
  background: rgba(241, 247, 255, 0.95);
  color: #2f5a8f;
  font-size: 14px;
  font-weight: 600;
  border-radius: 999px;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease;
}
.logout-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 24px rgba(136, 190, 247, 0.28);
}
</style>
