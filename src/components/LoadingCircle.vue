<template>
  <div class="loading-circle" role="status" aria-live="polite" aria-busy="true" aria-label="加载中">
    <svg class="loading-circle__svg" viewBox="0 0 50 50" aria-hidden="true">
      <circle class="loading-circle__path" cx="25" cy="25" r="20" fill="none" stroke-width="5" />
    </svg>
  </div>
</template>

<script setup lang="ts">
// Stateless loading spinner component
</script>

<style scoped>
.loading-circle {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: #2f5a8f;
}
.loading-circle__svg {
  width: 28px;
  height: 28px;
  animation: lc-rotate 1.2s linear infinite;
}
.loading-circle__path {
  stroke: #76b8ff;
  stroke-linecap: round;
  animation: lc-dash 1.6s ease-in-out infinite;
}
@keyframes lc-rotate {
  100% {
    transform: rotate(360deg);
  }
}
@keyframes lc-dash {
  0% {
    stroke-dasharray: 1, 150;
    stroke-dashoffset: 0;
  }
  50% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -35;
  }
  100% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -124;
  }
}
</style>
