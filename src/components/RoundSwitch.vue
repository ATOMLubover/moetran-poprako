<script setup lang="ts">
import { defineEmits } from 'vue';

// 支持 v-model 绑定：使用 props.modelValue + update:modelValue
const props = defineProps({ modelValue: { type: Boolean, default: false } });
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>();

function toggle() {
  emit('update:modelValue', !props.modelValue);
}
</script>
<template>
  <button class="round-switch" @click="toggle" :aria-pressed="props.modelValue">
    <span class="indicator" :class="{ 'indicator--on': props.modelValue }"></span>
  </button>
</template>
<style scoped>
.round-switch {
  width: 40px;
  height: 22px;
  border-radius: 20px;
  border: 1px solid rgba(150, 180, 210, 0.6);
  background: linear-gradient(135deg, #f5fafe, #eef6fd);
  cursor: pointer;
  position: relative;
  padding: 0;
  display: flex;
  align-items: center;
  transition:
    background 0.22s ease,
    box-shadow 0.22s ease;
}
.round-switch:hover {
  box-shadow: 0 4px 12px rgba(140, 180, 230, 0.24);
}
.indicator {
  width: 18px;
  height: 18px;
  border-radius: 6px;
  background: #9fb8d1;
  margin-left: 2px;
  transition:
    transform 0.22s cubic-bezier(0.2, 0.9, 0.2, 1),
    background 0.22s ease;
}
.indicator--on {
  transform: translateX(18px);
  background: #62b4ff;
}
</style>
