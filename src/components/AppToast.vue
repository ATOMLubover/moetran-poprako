<script setup lang="ts">
import { computed } from 'vue';

type ToastTone = 'success' | 'error';

const props = withDefaults(
  defineProps<{
    visible: boolean;
    message: string;
    tone?: ToastTone;
  }>(),
  {
    tone: 'success',
  }
);

const toneClass = computed(() =>
  props.tone === 'error' ? 'app-toast__body--error' : 'app-toast__body--success'
);
</script>

<template>
  <Transition name="app-toast">
    <div v-if="visible" class="app-toast" role="status" aria-live="polite" aria-atomic="true">
      <div class="app-toast__body" :class="toneClass">
        <slot>{{ message }}</slot>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.app-toast {
  position: fixed;
  left: 50%;
  top: -80px;
  transform: translate(-50%, 96px);
  z-index: 1200;
  pointer-events: none;
}

.app-toast__body {
  min-width: 220px;
  max-width: 360px;
  padding: 12px 20px;
  border-radius: 12px;
  color: #ffffff;
  font-size: 14px;
  font-weight: 500;
  text-align: center;
  box-shadow: 0 18px 36px rgba(24, 63, 102, 0.22);
  background: linear-gradient(135deg, #94e2a8 0%, #5bd259 100%);
}

.app-toast__body--success {
  background: linear-gradient(135deg, #94e2a8 0%, #5bd259 100%);
}

.app-toast__body--error {
  background: linear-gradient(135deg, #f37262 0%, #d9413b 100%);
}

.app-toast-enter-from,
.app-toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 16px);
}

.app-toast-enter-active,
.app-toast-leave-active {
  transition:
    opacity 0.26s ease,
    transform 0.26s ease;
}
</style>
