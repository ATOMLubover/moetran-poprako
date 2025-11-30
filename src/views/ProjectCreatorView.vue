<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useToastStore } from '../stores/toast';
import { useTokenStore } from '../stores/token';
// member search is now handled inside `MemberSelector` component
import {
  assignMemberToProj,
  createProj,
  getTeamPoprakoProjsets,
  type PoprakoProjsetInfo,
} from '../ipc/project';
import CircularProgress from '../components/CircularProgress.vue';
import MemberSelector from '../components/MemberSelector.vue';

// props：由 PanelView 传入当前团队 ID
// Props: 从父组件（PanelView）注入当前选中的团队 ID
const props = defineProps<{ teamId?: string | null }>();

// 邀请后保存的成员信息（仅保留 id 与 name，不含 position）
interface MemberInfo {
  id: string;
  name: string;
}

const emit = defineEmits<{ (e: 'close'): void }>();

// 作者名输入
const authorName = ref<string>('');

// 表单主体数据
const projectInfo = ref<{
  title: string;
  description: string;
  allowAutoJoin: boolean;
  isHidden: boolean;
  applicantMemberId: number;
  worksetId: number;
}>({
  title: '',
  description: '',
  allowAutoJoin: false,
  isHidden: false,
  applicantMemberId: 0,
  worksetId: 0,
});

// 提交状态与提示文案
const loading = ref<boolean>(false);
const message = ref<string>('');

// // 根据提示内容调整样式（成功 / 失败）
// const messageClass = computed(() => {
//   if (!message.value) {
//     return '';
//   }

//   return message.value.includes('成功') ? 'creator-message--success' : 'creator-message--error';
// });

// 最终标题预览（组装 [author]title）
const finalTitlePreview = computed(() => {
  if (!authorName.value || !projectInfo.value.title) {
    return '';
  }

  return `[${authorName.value}]${projectInfo.value.title}`;
});

// Stores: token
const toastStore = useToastStore();
const tokenStore = useTokenStore();
const { moetranToken } = storeToRefs(tokenStore);

// 当前团队下的项目集列表与选择
const selectedProjsetId = ref<string>('');
const currentProjsets = ref<PoprakoProjsetInfo[]>([]);
const projsetLoading = ref(false);

// 监听 teamId 变化，重置本地 projset 列表并自动拉取
watch(
  () => props.teamId,
  () => {
    selectedProjsetId.value = '';
    currentProjsets.value = [];
    projsetLoading.value = false;
    void loadProjsetsForCurrentTeam();
  },
  { immediate: true }
);

// Creator 打开后，由外层传入 teamId，此时加载一次 projset 列表
async function loadProjsetsForCurrentTeam(): Promise<void> {
  const teamId = props.teamId ?? '';
  if (!teamId || projsetLoading.value) return;

  projsetLoading.value = true;
  try {
    const list = await getTeamPoprakoProjsets(teamId);

    console.log('Loaded projsets for team', teamId, list);

    currentProjsets.value = list;
  } catch (e) {
    console.error('Failed to load projsets for team', teamId, e);
    toastStore.show('加载项目集失败，请稍后重试');
  } finally {
    projsetLoading.value = false;
  }
}

// 统一：每个角色一个数组，存储 MemberInfo { id, name }
const invitedTranslators = ref<MemberInfo[]>([]);
const invitedProofreaders = ref<MemberInfo[]>([]);
const invitedTypesetters = ref<MemberInfo[]>([]);

// Use the shared MemberSelector component: maintain a single picked list
// that includes position so the component can manage picks across roles.
const selectorOpen = ref(false);
const selectorRole = ref<'translator' | 'proofreader' | 'typesetter' | null>(null);
const pickedAll = ref<
  { id: string; name: string; position: 'translator' | 'proofreader' | 'typesetter' }[]
>([]);

function openSelector(role: 'translator' | 'proofreader' | 'typesetter'): void {
  selectorRole.value = role;
  // build pickedAll from existing invited lists
  pickedAll.value = [
    ...invitedTranslators.value.map(m => ({
      id: m.id,
      name: m.name,
      position: 'translator' as const,
    })),
    ...invitedProofreaders.value.map(m => ({
      id: m.id,
      name: m.name,
      position: 'proofreader' as const,
    })),
    ...invitedTypesetters.value.map(m => ({
      id: m.id,
      name: m.name,
      position: 'typesetter' as const,
    })),
  ];
  selectorOpen.value = true;
}

function closeSelector(): void {
  selectorOpen.value = false;
  selectorRole.value = null;
  pickedAll.value = [];
}

function onMemberSelectorConfirm(): void {
  // split pickedAll back into per-role arrays
  invitedTranslators.value = pickedAll.value
    .filter(p => p.position === 'translator')
    .map(p => ({ id: p.id, name: p.name }));
  invitedProofreaders.value = pickedAll.value
    .filter(p => p.position === 'proofreader')
    .map(p => ({ id: p.id, name: p.name }));
  invitedTypesetters.value = pickedAll.value
    .filter(p => p.position === 'typesetter')
    .map(p => ({ id: p.id, name: p.name }));

  closeSelector();
}

function onMemberSelectorCancel(): void {
  // MemberSelector will have restored pickedAll via props mutation; reflect into invited arrays
  invitedTranslators.value = pickedAll.value
    .filter(p => p.position === 'translator')
    .map(p => ({ id: p.id, name: p.name }));
  invitedProofreaders.value = pickedAll.value
    .filter(p => p.position === 'proofreader')
    .map(p => ({ id: p.id, name: p.name }));
  invitedTypesetters.value = pickedAll.value
    .filter(p => p.position === 'typesetter')
    .map(p => ({ id: p.id, name: p.name }));

  closeSelector();
}

// Note: permission toggles UI was removed; keep fields in projectInfo for future use

// 关闭当前创建视图
function handleClose(): void {
  emit('close');
}

// 表单提交：创建项目
// 提交表单：创建项目（包含前端必要校验）
async function handleCreateProject(): Promise<void> {
  console.log('handleCreateProject invoked', {
    teamId: props.teamId,
    title: projectInfo.value.title,
  });
  if (!authorName.value || !projectInfo.value.title || !projectInfo.value.description) {
    toastStore.show('请完整填写作者、标题和描述');

    return;
  }

  if (!props.teamId) {
    toastStore.show('请先在左侧选择一个汉化组');

    return;
  }

  if (!selectedProjsetId.value) {
    toastStore.show('请选择一个项目集');

    return;
  }

  if (!moetranToken.value) {
    toastStore.show('缺少 Moetran Token，请先登录');

    return;
  }

  loading.value = true;
  message.value = '';

  try {
    const projName = finalTitlePreview.value || projectInfo.value.title;

    const created = await createProj({
      projName: projName,
      projDescription: projectInfo.value.description,
      teamId: props.teamId,
      projsetId: selectedProjsetId.value,
      mtrAuth: moetranToken.value,
      worksetIndex: projectInfo.value.worksetId,
      sourceLanguage: 'ja',
      targetLanguages: ['zh-CN'],
      allowApplyType: projectInfo.value.allowAutoJoin ? 1 : 0,
      applicationCheckType: 0,
      defaultRole: '63d87c24b8bebd75ff934267',
    });

    const projId = created.projId;

    const allInvites: { id: string; role: 'translator' | 'proofreader' | 'typesetter' }[] = [
      ...invitedTranslators.value.map(m => ({ id: m.id, role: 'translator' as const })),
      ...invitedProofreaders.value.map(m => ({ id: m.id, role: 'proofreader' as const })),
      ...invitedTypesetters.value.map(m => ({ id: m.id, role: 'typesetter' as const })),
    ];

    // 按照异常（reject）来检测每个指派操作的失败，并收集失败明细以便展示或重试
    const assignResults = await Promise.all(
      allInvites.map(async invite => {
        try {
          await assignMemberToProj({
            projId: projId,
            memberId: invite.id,
            isTranslator: invite.role === 'translator',
            isProofreader: invite.role === 'proofreader',
            isTypesetter: invite.role === 'typesetter',
            isPrincipal: false,
          });

          return { invite, ok: true } as const;
        } catch (err) {
          return { invite, ok: false, error: err } as const;
        }
      })
    );

    const failedDetails = assignResults.filter(r => !r.ok) as Array<{
      invite: { id: string; role: 'translator' | 'proofreader' | 'typesetter' };
      ok: false;
      error: unknown;
    }>;

    if (failedDetails.length > 0) {
      const brief = failedDetails
        .map(f => `${f.invite.id}(${f.invite.role}): ${String(f.error)}`)
        .join('；');

      message.value = `项目创建成功，但 ${failedDetails.length} 个成员指派失败：${brief}`;
      toastStore.show(message.value);
    } else {
      message.value = '项目创建成功，并已指派预邀请成员';
      toastStore.show('项目创建成功');
    }
  } catch (err) {
    console.error('Create project failed', err);
    message.value = `项目创建失败：${String(err)}`;
    toastStore.show('项目创建失败，请稍后重试');
  } finally {
    loading.value = false;
  }
}
</script>
<template>
  <section class="creator-root">
    <header class="creator-header">
      <div class="creator-header__left">
        <h1 class="creator-title">创建新项目</h1>
      </div>
      <div class="creator-header__right">
        <button type="button" class="creator-close" @click="handleClose" title="关闭">×</button>
      </div>
    </header>

    <div class="creator-body">
      <div v-if="!props.teamId" class="creator-no-team">
        请先在左侧选择一个汉化组（团队），以加载项目集并启用成员搜索。
      </div>
      <form class="creator-form" @submit.prevent="handleCreateProject">
        <div class="creator-row">
          <div class="creator-field">
            <label for="creator-author" class="creator-label">作者</label>
            <input
              id="creator-author"
              v-model="authorName"
              type="text"
              required
              placeholder="作者名称"
              class="creator-input"
            />
          </div>

          <div class="creator-field">
            <label for="creator-title" class="creator-label">标题</label>
            <input
              id="creator-title"
              v-model="projectInfo.title"
              type="text"
              required
              placeholder="无须输入序号"
              class="creator-input"
            />
            <div v-if="finalTitlePreview" class="creator-title-preview">
              <span class="creator-title-preview__label">最终标题预览：</span>
              <span class="creator-title-preview__text">{{ finalTitlePreview }}</span>
            </div>
          </div>
        </div>

        <div class="creator-field">
          <label class="creator-label">所属项目集</label>
          <select
            v-model="selectedProjsetId"
            class="creator-input"
            :disabled="!props.teamId"
            @focus="loadProjsetsForCurrentTeam"
            @click="loadProjsetsForCurrentTeam"
          >
            <option value="" disabled>请选择项目集</option>
            <option v-for="ps in currentProjsets" :key="ps.projsetId" :value="ps.projsetId">
              {{ ps.projsetName }}
            </option>
          </select>
          <div v-if="projsetLoading" class="creator-projset-loading">
            <CircularProgress :progress="50" :size="16" />
            <span class="creator-projset-loading__text">正在加载项目集...</span>
          </div>
        </div>

        <div class="creator-field">
          <label for="creator-desc" class="creator-label">描述</label>
          <textarea
            id="creator-desc"
            v-model="projectInfo.description"
            rows="1"
            required
            placeholder="项目描述"
            class="creator-textarea"
          ></textarea>
        </div>

        <!-- <div class="creator-field">
          <div class="creator-label">权限控制</div>
          <div class="creator-perms">
            <button
              type="button"
              class="creator-perm-btn"
              :class="{ 'creator-perm-btn--active': projectInfo.allowAutoJoin }"
              @click="handleToggleAllowAutoJoin"
            >
              允许自动加入
            </button>
            <button
              type="button"
              class="creator-perm-btn"
              :class="{ 'creator-perm-btn--active': projectInfo.isHidden }"
              @click="handleToggleHidden"
            >
              设为隐藏
            </button>
          </div>
        </div>
        <div v-if="message" :class="['creator-message', messageClass]">
          {{ message }}
        </div> -->

        <!-- 隐藏提交按钮以支持回车提交表单 -->
        <button type="submit" style="display: none" aria-hidden="true"></button>
      </form>

      <!-- 预邀请成员展示与邀请入口 -->
      <div class="creator-invite-block">
        <div class="creator-label">预邀请成员</div>

        <div class="creator-invite-item">
          <div class="creator-invite-text">
            翻译：
            <span class="creator-invite-names">
              {{
                invitedTranslators.length ? invitedTranslators.map(m => m.name).join('、') : '暂无'
              }}
            </span>
          </div>
          <button type="button" class="creator-invite-btn" @click="openSelector('translator')">
            邀请
          </button>
        </div>

        <div class="creator-invite-item">
          <div class="creator-invite-text">
            校对：
            <span class="creator-invite-names">
              {{
                invitedProofreaders.length
                  ? invitedProofreaders.map(m => m.name).join('、')
                  : '暂无'
              }}
            </span>
          </div>
          <button type="button" class="creator-invite-btn" @click="openSelector('proofreader')">
            邀请
          </button>
        </div>

        <div class="creator-invite-item">
          <div class="creator-invite-text">
            嵌字：
            <span class="creator-invite-names">
              {{
                invitedTypesetters.length ? invitedTypesetters.map(m => m.name).join('、') : '暂无'
              }}
            </span>
          </div>
          <button type="button" class="creator-invite-btn" @click="openSelector('typesetter')">
            邀请
          </button>
        </div>
      </div>

      <!-- Shared MemberSelector component -->
      <MemberSelector
        :show="selectorOpen"
        :picked="pickedAll"
        :team-id="props.teamId ?? undefined"
        :initial-role="selectorRole ?? undefined"
        @confirm="onMemberSelectorConfirm"
        @cancel="onMemberSelectorCancel"
        @close="closeSelector"
      />
    </div>
    <!-- 底部固定提交区域 -->
    <div class="creator-footer">
      <button
        type="button"
        class="creator-submit fb-confirm-btn"
        :disabled="loading"
        @click="handleCreateProject"
      >
        {{ loading ? '正在创建...' : '确认创建' }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.creator-root {
  display: flex;
  flex-direction: column;
  padding: 24px 28px 32px;
  box-sizing: border-box;
  height: 100%;
  background: linear-gradient(180deg, #f5f9ff 0%, #ffffff 65%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
}

.creator-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
}

.creator-header__left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.creator-title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.4px;
}

.creator-subtitle {
  margin: 0;
  font-size: 13px;
  color: #4a5f7a;
}

.creator-header__right {
  display: flex;
  align-items: center;
}

.creator-close {
  border: 1px solid rgba(150, 180, 210, 0.5);
  background: #fff;
  color: #2a3d52;
  width: 32px;
  height: 32px;
  border-radius: 9px;
  cursor: pointer;
  font-size: 18px;
  line-height: 30px;
  padding: 0;
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.creator-close:hover {
  box-shadow: 0 8px 20px rgba(140, 180, 230, 0.25);
  transform: translateY(-2px);
}

.creator-body {
  margin-top: 22px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.creator-no-team {
  padding: 10px 12px;
  border-radius: 8px;
  background: #fff6e6;
  color: #6b4a00;
  border: 1px solid rgba(220, 180, 110, 0.6);
  font-size: 13px;
}

.creator-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.creator-row {
  display: flex;
  gap: 12px;
}
.creator-row .creator-field {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.creator-footer {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
  padding: 12px 28px 18px;
  border-top: 1px solid rgba(170, 190, 215, 0.6);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0), rgba(250, 250, 250, 0.03));
}

.creator-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.creator-label {
  font-size: 14px;
  font-weight: 600;
  color: #23415b;
}

.creator-input {
  width: 100%;
  padding: 8px 11px;
  border-radius: 10px;
  border: 1px solid rgba(170, 190, 215, 0.9);
  font-size: 14px;
  box-sizing: border-box;
  outline: none;
  background: #ffffff;
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.creator-input:focus {
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 0 0 1px rgba(118, 184, 255, 0.55);
}

.creator-textarea {
  min-width: 130px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.2px;
  cursor: pointer;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.06);
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    background 0.12s ease;
}

/* Submit and invite button styles - unified light, non-gradient look */
.creator-submit {
  min-width: 130px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.2px;
  cursor: pointer;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.06);
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    background 0.12s ease;
}

.creator-submit:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.12);
  background: #eef6ff;
}

.creator-submit:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
}

.creator-perm-btn--active {
  background: #e8f5ff;
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 8px 20px rgba(136, 190, 247, 0.35);
}

.creator-actions {
  margin-top: 6px;
  display: flex;
  justify-content: flex-end;
}

.creator-title-preview__label {
  font-weight: 600;
  margin-right: 4px;
}

.creator-title-preview__text {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.creator-perms {
  display: flex;
  flex-direction: row;
  gap: 10px;
}

.creator-invite-block {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.creator-invite-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 12px;
  border-radius: 12px;
  background: #f6f8fc;
  border: 1px solid rgba(170, 190, 215, 0.9);
}

.creator-invite-text {
  font-size: 13px;
  color: #23415b;
}

.creator-invite-names {
  margin-left: 4px;
  color: #4a5f7a;
}

.creator-invite-btn {
  padding: 7px 12px;
  border-radius: 10px;
  border: 1px solid rgba(118, 184, 255, 0.28);
  background: #f4f9ff;
  color: #2f5a8f;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 6px 14px rgba(118, 184, 255, 0.05);
  transition:
    transform 0.14s ease,
    box-shadow 0.14s ease,
    background 0.1s ease;
}

.creator-invite-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 20px rgba(118, 184, 255, 0.08);
  background: #eef6ff;
}

/* MemberSelector 悬浮窗样式（示例） */
.selector-overlay {
  position: fixed;
  inset: 0;
  background: rgba(10, 20, 40, 0.22);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.selector-panel {
  width: 360px;
  max-width: calc(100% - 40px);
  background: #ffffff;
  border-radius: 16px;
  box-shadow: 0 18px 40px rgba(40, 70, 120, 0.45);
  padding: 16px 18px 18px;
  box-sizing: border-box;
}

.selector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.selector-title {
  font-size: 15px;
  font-weight: 600;
  color: #203650;
}

.selector-close {
  border: none;
  background: transparent;
  font-size: 18px;
  cursor: pointer;
  color: #4a5f7a;
}

.selector-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.selector-search {
  display: flex;
  gap: 8px;
}

.selector-input {
  flex: 1;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(170, 190, 215, 0.9);
  font-size: 13px;
}

.selector-search-btn {
  padding: 8px 14px;
  border-radius: 999px;
  border: none;
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #ffffff;
  font-size: 13px;
  cursor: pointer;
}

.selector-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 220px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.selector-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 7px 9px;
  border-radius: 8px;
  background: #f6f8fc;
  cursor: pointer;
  font-size: 13px;
  border: 1px solid transparent;
  transition:
    background 0.16s ease,
    border-color 0.16s ease,
    box-shadow 0.16s ease;
}

.selector-item:hover {
  background: #e6f1ff;
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.5);
}

.selector-item__name {
  font-weight: 500;
}

.selector-item__id {
  font-size: 12px;
  color: #6b7c91;
}

.selector-item--picked {
  background: #e4f7eb;
  border-color: rgba(120, 200, 150, 0.9);
  box-shadow: none;
}

.selector-item--picked:hover {
  background: #e4f7eb;
  border-color: rgba(120, 200, 150, 0.9);
  box-shadow: none;
}

.selector-empty {
  text-align: center;
  font-size: 13px;
  color: #6b7c91;
  padding: 16px 0 6px;
}

.selector-picked {
  margin-top: 6px;
  font-size: 12px;
  color: #3a4d6a;
}

.selector-picked-label {
  font-weight: 600;
}

.selector-picked-names {
  margin-left: 4px;
}

.selector-actions {
  margin-top: 10px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.selector-action-btn {
  min-width: 72px;
  padding: 6px 12px;
  border-radius: 999px;
  border: none;
  font-size: 13px;
  cursor: pointer;
}

.selector-action-btn--cancel {
  background: #f3f5f9;
  color: #4a5f7a;
}

.selector-action-btn--confirm {
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #ffffff;
}

.selector-icon-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.selector-icon-btn--remove {
  color: #c04a4a;
}

.selector-icon-btn--add {
  color: #2f7ad1;
}

@media (max-width: 960px) {
  .creator-root {
    padding: 20px 18px 28px;
  }
}

/* 针对低高度屏幕进行适配，减小间距和字号 */
@media (max-height: 760px) {
  .creator-root {
    padding: 12px 14px 12px;
  }
  .creator-title {
    font-size: 18px;
  }
  .creator-label {
    font-size: 13px;
  }
  .creator-input,
  .creator-textarea {
    padding: 6px 8px;
    font-size: 13px;
  }
  .creator-submit {
    padding: 8px 14px;
    font-size: 13px;
  }
  .creator-body {
    gap: 8px;
  }
}
</style>
