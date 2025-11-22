import { createApp } from 'vue';
import App from './App.vue';
import { createPinia } from 'pinia';

// 创建 Pinia 实例并挂载，避免使用 store 时白屏
const pinia = createPinia();

createApp(App).use(pinia).mount('#app');
