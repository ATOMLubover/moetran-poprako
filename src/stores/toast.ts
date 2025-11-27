import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastTone = 'success' | 'error';

export const useToastStore = defineStore('toast', () => {
  const visible = ref(false);
  const message = ref('');
  const tone = ref<ToastTone>('success');
  let timer: number | null = null;

  function show(msg: string, t: ToastTone = 'success', duration = 2400): void {
    message.value = msg;
    tone.value = t;
    visible.value = true;
    if (timer !== null) {
      window.clearTimeout(timer);
    }
    timer = window.setTimeout(() => {
      visible.value = false;
      timer = null;
    }, duration);
  }

  function hide(): void {
    if (timer !== null) {
      window.clearTimeout(timer);
      timer = null;
    }
    visible.value = false;
  }

  return { visible, message, tone, show, hide };
});
