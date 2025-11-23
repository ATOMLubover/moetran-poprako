<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useToastStore } from '../stores/toast';
import { useTokenStore } from '../stores/token';
import { getUserInfo } from '../ipc/user';
import { getUserTeams } from '../ipc/team';
import { getUserProjects } from '../ipc/project';
import type { ResUser } from '../api/model/user';
import type { ResTeam } from '../api/model/team';
import type { ResProject } from '../api/model/project';

// 仪表盘页面：展示用户信息 + 汉化组列表 + 项目列表

// 用户信息
const user = ref<ResUser | null>(null);

// 汉化组列表（左侧 1/3 区域展示）
const teams = ref<ResTeam[]>([]);
const teamsPage = ref<number>(1);
const teamsLimit = ref<number>(8);

// 项目列表（主体内容核心区域）
const projects = ref<ResProject[]>([]);
const projectsPage = ref<number>(1);
const projectsLimit = ref<number>(10);

// 加载状态
const loadingUser = ref<boolean>(false);
const loadingTeams = ref<boolean>(false);
const loadingProjects = ref<boolean>(false);

// 依赖的 store
const toastStore = useToastStore();
const tokenStore = useTokenStore();

// 载入用户信息
async function loadUser(): Promise<void> {
  loadingUser.value = true;

  try {
    user.value = await getUserInfo();
  } catch (err) {
    toastStore.show('获取用户信息失败');
  }

  loadingUser.value = false;
}

// 载入汉化组列表
async function loadTeams(): Promise<void> {
  loadingTeams.value = true;

  try {
    teams.value = await getUserTeams({ page: teamsPage.value, limit: teamsLimit.value });
  } catch (err) {
    toastStore.show('获取汉化组列表失败');
  }

  loadingTeams.value = false;
}

// 载入项目列表
async function loadProjects(): Promise<void> {
  loadingProjects.value = true;

  try {
    projects.value = await getUserProjects({
      page: projectsPage.value,
      limit: projectsLimit.value,
    });
  } catch (err) {
    toastStore.show('获取项目列表失败');
  }

  loadingProjects.value = false;
}

// 切换分页（团队）
function handleTeamsPrev(): void {
  if (teamsPage.value <= 1) return;

  teamsPage.value -= 1;
  loadTeams();
}

function handleTeamsNext(): void {
  teamsPage.value += 1;
  loadTeams();
}

// 切换分页（项目）
function handleProjectsPrev(): void {
  if (projectsPage.value <= 1) return;

  projectsPage.value -= 1;
  loadProjects();
}

function handleProjectsNext(): void {
  projectsPage.value += 1;
  loadProjects();
}

// 初始化加载（仅在拥有 moetran token 时进行）
onMounted(() => {
  if (!tokenStore.moetranToken) {
    toastStore.show('尚未登录，无法载入仪表盘');
    return;
  }

  loadUser();
  loadTeams();
  loadProjects();
});
</script>

<template>
  <div class="panel-root">
    <!-- 顶部用户信息（左上角） -->
    <div class="user-info" v-if="user">
      <div class="user-name">{{ user.name }}</div>
      <div class="user-id">ID: {{ user.id.slice(0, 8) }}...</div>
    </div>
    <div v-else class="user-info user-info--loading">
      <span v-if="loadingUser">载入用户...</span>
      <span v-else>无用户信息</span>
    </div>

    <!-- 左侧汉化组列表区域 -->
    <aside class="teams-sidebar">
      <div class="teams-header">
        <h2 class="teams-title">我的汉化组</h2>
        <div class="teams-pagination">
          <button
            type="button"
            class="nav-btn"
            @click="handleTeamsPrev"
            :disabled="teamsPage === 1"
          >
            ◀
          </button>
          <span class="page-indicator">{{ teamsPage }}</span>
          <button type="button" class="nav-btn" @click="handleTeamsNext">▶</button>
        </div>
      </div>
      <div class="teams-list">
        <div v-if="loadingTeams" class="placeholder">载入汉化组...</div>
        <template v-else>
          <div v-for="team in teams" :key="team.id" class="team-item">
            <div class="team-name">{{ team.name }}</div>
          </div>
          <div v-if="teams.length === 0" class="placeholder">暂无汉化组</div>
        </template>
      </div>
    </aside>

    <!-- 主体项目列表区域 -->
    <main class="projects-main">
      <div class="projects-header">
        <h2 class="projects-title">参与的项目</h2>
        <div class="projects-pagination">
          <button
            type="button"
            class="nav-btn"
            @click="handleProjectsPrev"
            :disabled="projectsPage === 1"
          >
            ◀
          </button>
          <span class="page-indicator">{{ projectsPage }}</span>
          <button type="button" class="nav-btn" @click="handleProjectsNext">▶</button>
        </div>
      </div>
      <div class="projects-grid">
        <div v-if="loadingProjects" class="placeholder">载入项目...</div>
        <template v-else>
          <div v-for="project in projects" :key="project.id" class="project-card">
            <div class="project-header">
              <div class="project-name">{{ project.name }}</div>
            </div>
            <div class="project-stats">
              <div class="stat-block">
                <span class="stat-label">源文件</span>
                <span class="stat-value">{{ project.source_count }}</span>
              </div>
              <div class="stat-block">
                <span class="stat-label">已翻译</span>
                <span class="stat-value">{{ project.translated_source_count }}</span>
              </div>
              <div class="stat-block">
                <span class="stat-label">已校对</span>
                <span class="stat-value">{{ project.checked_source_count }}</span>
              </div>
            </div>
            <!-- 预留扩展区域：未来可放置 Poprako / Moetran 混合信息 -->
            <div class="project-extra"></div>
          </div>
          <div v-if="projects.length === 0" class="placeholder">暂无项目</div>
        </template>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* 根容器布局：左侧固定宽度栏 + 右侧自适应主体 */
.panel-root {
  position: relative;
  display: flex;
  flex-direction: row;
  width: 100%;
  min-height: 100vh;
  box-sizing: border-box;
  background: linear-gradient(180deg, #f4f9ff 0%, #ffffff 70%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
  padding: 18px 22px;
  gap: 24px;
}

/* 用户信息展示（左上角） */
.user-info {
  position: absolute;
  top: 14px;
  left: 22px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(150, 180, 210, 0.4);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 13px;
  box-shadow: 0 4px 16px rgba(140, 180, 230, 0.15);
  backdrop-filter: blur(4px);
}
.user-info--loading {
  opacity: 0.7;
}
.user-name {
  font-weight: 600;
}
.user-id {
  margin-top: 4px;
  color: #3d566f;
  font-size: 11px;
}

/* 左侧汉化组列表区域 */
.teams-sidebar {
  flex: 0 0 26%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 14px;
  padding: 16px 16px 18px;
  box-sizing: border-box;
  box-shadow: 0 8px 28px rgba(140, 180, 230, 0.18);
  height: calc(100vh - 36px);
}
.teams-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.teams-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}
.teams-pagination {
  display: flex;
  align-items: center;
  gap: 6px;
}
.page-indicator {
  font-size: 13px;
  color: #32526d;
}
.nav-btn {
  border: 1px solid rgba(118, 184, 255, 0.7);
  background: #f1f7ff;
  color: #2f5a8f;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 7px;
  cursor: pointer;
  transition:
    background 0.16s ease,
    transform 0.16s ease;
}
.nav-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
.nav-btn:not(:disabled):hover {
  background: #e4f1ff;
  transform: translateY(-2px);
}
.teams-list {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}
.team-item {
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 500;
  color: #27445c;
  display: flex;
  align-items: center;
  justify-content: space-between;
  transition:
    box-shadow 0.16s ease,
    transform 0.16s ease;
}
.team-item:hover {
  box-shadow: 0 6px 18px rgba(136, 190, 247, 0.25);
  transform: translateY(-2px);
}

/* 主体项目列表区域 */
.projects-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border: 1px solid rgba(150, 180, 210, 0.3);
  border-radius: 18px;
  padding: 20px 22px 26px;
  box-sizing: border-box;
  box-shadow: 0 10px 34px rgba(140, 180, 230, 0.22);
  height: calc(100vh - 36px);
}
.projects-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.projects-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}
.projects-pagination {
  display: flex;
  align-items: center;
  gap: 6px;
}
.projects-grid {
  margin-top: 18px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 18px 18px;
  flex: 1;
}
.project-card {
  background: #fff;
  border: 1px solid rgba(150, 180, 210, 0.35);
  border-radius: 14px;
  padding: 14px 16px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  position: relative;
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease;
}
.project-card:hover {
  box-shadow: 0 10px 30px rgba(136, 190, 247, 0.28);
  transform: translateY(-3px);
}
.project-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.project-name {
  font-size: 15px;
  font-weight: 600;
  color: #23415b;
}
.project-stats {
  display: flex;
  flex-direction: row;
  gap: 12px;
  flex-wrap: wrap;
}
.stat-block {
  display: flex;
  flex-direction: column;
  min-width: 70px;
}
.stat-label {
  font-size: 11px;
  color: #5b7a95;
}
.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: #2b526d;
}
.project-extra {
  height: 14px;
  border-top: 1px dashed rgba(150, 180, 210, 0.4);
}

/* 占位符 */
.placeholder {
  font-size: 13px;
  color: #6b859d;
}
</style>
