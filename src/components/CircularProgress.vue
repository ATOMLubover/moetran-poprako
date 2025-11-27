<script setup lang="ts">
import { computed } from 'vue';

// 简易圆形进度条组件，保留可扩展空间
interface Props {
  progress: number; // 0-100
  color?: 'yellow' | 'pink';
  label?: string;
  size?: number; // 直径 px
  stroke?: number; // 线宽 px
}

const props = withDefaults(defineProps<Props>(), {
  color: 'yellow',
  label: '',
  size: 110,
  stroke: 10,
});

const radius = computed(() => (props.size - props.stroke) / 2);
const circumference = computed(() => 2 * Math.PI * radius.value);
const clamped = computed(() => Math.min(100, Math.max(0, props.progress)));
const dashOffset = computed(() => circumference.value * (1 - clamped.value / 100));

const colorMap: Record<string, { track: string; bar: string; text: string }> = {
  yellow: { track: '#f3e8c7', bar: '#d9a425', text: '#b98119' },
  pink: { track: '#f9d5e4', bar: '#d35d92', text: '#b24874' },
};

const palette = computed(() => colorMap[props.color] || colorMap.yellow);
</script>

<template>
  <div class="cp-root" :style="{ width: size + 'px' }">
    <div class="cp-svg-wrapper" :style="{ width: size + 'px', height: size + 'px' }">
      <svg :width="size" :height="size" class="cp-svg">
        <circle
          class="cp-track"
          :cx="size / 2"
          :cy="size / 2"
          :r="radius"
          :stroke-width="stroke"
          :stroke="palette.track"
          fill="none"
        />
        <circle
          class="cp-bar"
          :cx="size / 2"
          :cy="size / 2"
          :r="radius"
          :stroke-width="stroke"
          :stroke="palette.bar"
          fill="none"
          :stroke-dasharray="circumference + ' ' + circumference"
          :stroke-dashoffset="dashOffset"
          stroke-linecap="round"
          :transform="'rotate(-90 ' + size / 2 + ' ' + size / 2 + ')'"
        />
        <text
          class="cp-text"
          :x="size / 2"
          :y="size / 2 + 4"
          text-anchor="middle"
          :fill="palette.text"
        >
          {{ clamped.toFixed(1) }}%
        </text>
      </svg>
    </div>
    <div v-if="label" class="cp-label">{{ label }}</div>
  </div>
</template>

<style scoped>
.cp-root {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.cp-svg-wrapper {
  position: relative;
}
.cp-svg {
  display: block;
}
.cp-track {
  opacity: 0.7;
}
.cp-bar {
  transition: stroke-dashoffset 0.6s ease;
}
.cp-text {
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 0.5px;
}
.cp-label {
  font-size: 12px;
  font-weight: 600;
  color: #445b72;
  letter-spacing: 0.4px;
}
</style>
