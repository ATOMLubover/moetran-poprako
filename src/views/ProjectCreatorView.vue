<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useToastStore } from '../stores/toast';
import { useTokenStore } from '../stores/token';
import { useProjsetStore } from '../stores/projset';
import type { ResMemberBrief, MemberPosition } from '../ipc/member';
import { searchMembersByName } from '../ipc/member';
import { assignMemberToProj, createProj } from '../ipc/project';

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

// 根据提示内容调整样式（成功 / 失败）
const messageClass = computed(() => {
  if (!message.value) {
    return '';
  }

  return message.value.includes('成功') ? 'creator-message--success' : 'creator-message--error';
});

// 最终标题预览（组装 [author]title）
const finalTitlePreview = computed(() => {
  if (!authorName.value || !projectInfo.value.title) {
    return '';
  }

  return `[${authorName.value}]${projectInfo.value.title}`;
});

// Stores: token 与 projset 缓存
const toastStore = useToastStore();
const tokenStore = useTokenStore();
const projsetStore = useProjsetStore();
const { moetranToken } = storeToRefs(tokenStore);

// 当前团队下的项目集列表与选择
const selectedProjsetId = ref<string>('');

// 计算属性：当前 props.teamId 对应的项目集列表
const currentProjsets = computed(() => {
  const teamId = props.teamId ?? '';
  return teamId ? projsetStore.getForTeam(teamId) : [];
});

// 监听 teamId 变化，触发项目集加载
watch(
  () => props.teamId,
  teamId => {
    selectedProjsetId.value = '';

    if (teamId) {
      void projsetStore.loadForTeam(teamId);
    }
  },
  { immediate: true }
);

// 统一：每个角色一个数组，存储 MemberInfo { id, name }
const invitedTranslators = ref<MemberInfo[]>([]);
const invitedProofreaders = ref<MemberInfo[]>([]);
const invitedTypesetters = ref<MemberInfo[]>([]);

// MemberSelector 悬浮窗状态
type InviteRole = 'translator' | 'proofreader' | 'typesetter' | null;
const selectorOpen = ref(false);
const selectorRole = ref<InviteRole>(null);
const selectorKeyword = ref('');
const selectorLoading = ref(false);
const selectorResults = ref<ResMemberBrief[]>([]);
// 本次打开选择器前的原始已选集合，用于取消时回滚（深拷贝）
let selectorInitialPicked: MemberInfo[] = [];

// 打开成员选择器：记录初始选择并展示已选成员
function openSelector(role: InviteRole): void {
  selectorRole.value = role;
  selectorKeyword.value = '';
  selectorResults.value = [];
  selectorOpen.value = true;

  // 记录打开时的初始选择，用于取消时恢复
  if (role === 'translator') {
    selectorInitialPicked = invitedTranslators.value.map(m => ({ ...m }));
  } else if (role === 'proofreader') {
    selectorInitialPicked = invitedProofreaders.value.map(m => ({ ...m }));
  } else if (role === 'typesetter') {
    selectorInitialPicked = invitedTypesetters.value.map(m => ({ ...m }));
  } else {
    selectorInitialPicked = [];
  }

  // 初次打开时立即展示当前已选成员（无须搜索）
  if (role) {
    const picked =
      role === 'translator'
        ? invitedTranslators.value
        : role === 'proofreader'
          ? invitedProofreaders.value
          : invitedTypesetters.value;

    selectorResults.value = picked.map(m => ({
      member_id: m.id,
      username: m.name,
    }));
  }
}

function closeSelector(): void {
  selectorOpen.value = false;
  selectorRole.value = null;
  selectorKeyword.value = '';
  selectorResults.value = [];
  selectorInitialPicked = [];
}

// 搜索成员（调用后端），避免重复 id 并合并已选成员
async function handleSearchMembers(): Promise<void> {
  selectorLoading.value = true;

  console.log(
    'Searching members for role:',
    selectorRole.value,
    'with keyword:',
    selectorKeyword.value
  );

  try {
    if (!selectorRole.value || !props.teamId) {
      selectorResults.value = [];

      console.warn('Member search aborted: role or teamId is missing');
    } else {
      // 当前角色已选成员
      const pickedIds: string[] =
        selectorRole.value === 'translator'
          ? invitedTranslators.value.map(m => m.id)
          : selectorRole.value === 'proofreader'
            ? invitedProofreaders.value.map(m => m.id)
            : invitedTypesetters.value.map(m => m.id);

      const pickedMembers: ResMemberBrief[] = selectorResults.value.filter(m =>
        pickedIds.includes(m.member_id)
      );

      const keyword = selectorKeyword.value.trim();

      if (!keyword) {
        // 无关键词：只显示已选成员
        selectorResults.value = pickedMembers;
        return;
      }

      const results = await searchMembersByName({
        team_id: props.teamId,
        position: selectorRole.value as MemberPosition,
        fuzzy_name: keyword,
        page: 1,
        limit: 20,
      });

      const pickedIdSet = new Set(pickedMembers.map(m => m.member_id));

      const filteredResults = results.filter(m => !pickedIdSet.has(m.member_id));

      selectorResults.value = [...pickedMembers, ...filteredResults];
    }
  } finally {
    selectorLoading.value = false;
  }
}

function handleSelectMember(memberId: string): void {
  const member = selectorResults.value.find(m => m.member_id === memberId);
  if (!member) return;

  const targetArr =
    selectorRole.value === 'translator'
      ? invitedTranslators
      : selectorRole.value === 'proofreader'
        ? invitedProofreaders
        : invitedTypesetters;

  if (!targetArr.value.some(m => m.id === member.member_id)) {
    targetArr.value.push({ id: member.member_id, name: member.username });
  }

  // 无关键词时实时刷新展示已选成员
  if (!selectorKeyword.value.trim()) {
    void handleSearchMembers();
  }
}

function handleRemoveMember(memberId: string): void {
  const targetArr =
    selectorRole.value === 'translator'
      ? invitedTranslators
      : selectorRole.value === 'proofreader'
        ? invitedProofreaders
        : invitedTypesetters;
  targetArr.value = targetArr.value.filter(m => m.id !== memberId);

  // 无关键词时实时刷新展示已选成员
  if (!selectorKeyword.value.trim()) {
    void handleSearchMembers();
  }
}

function handleConfirmSelector(): void {
  selectorOpen.value = false;
  selectorRole.value = null;
  selectorKeyword.value = '';
  selectorResults.value = [];
  selectorInitialPicked = [];
}

function handleCancelSelector(): void {
  // 恢复到打开选择器前的状态
  if (selectorRole.value === 'translator') {
    invitedTranslators.value = selectorInitialPicked.map(m => ({ ...m }));
  } else if (selectorRole.value === 'proofreader') {
    invitedProofreaders.value = selectorInitialPicked.map(m => ({ ...m }));
  } else if (selectorRole.value === 'typesetter') {
    invitedTypesetters.value = selectorInitialPicked.map(m => ({ ...m }));
  }
  closeSelector();
}

// 权限字段联动：允许自动加入时，不能是隐藏项目
function handleToggleAllowAutoJoin(): void {
  projectInfo.value.allowAutoJoin = !projectInfo.value.allowAutoJoin;

  if (projectInfo.value.allowAutoJoin) {
    projectInfo.value.isHidden = false;
  }
}

// 权限字段联动：隐藏项目时，不能允许自动加入
function handleToggleHidden(): void {
  projectInfo.value.isHidden = !projectInfo.value.isHidden;

  if (projectInfo.value.isHidden) {
    projectInfo.value.allowAutoJoin = false;
  }
}

// 关闭当前创建视图
function handleClose(): void {
  emit('close');
}

// 表单提交：创建项目
// 提交表单：创建项目（包含前端必要校验）
async function handleCreateProject(): Promise<void> {
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
      proj_name: projName,
      proj_description: projectInfo.value.description,
      team_id: props.teamId,
      projset_id: selectedProjsetId.value,
      mtr_auth: moetranToken.value,
      workset_index: projectInfo.value.worksetId,
      source_language: 'ja',
      target_languages: ['zh-CN'],
      allow_apply_type: projectInfo.value.allowAutoJoin ? 1 : 0,
      application_check_type: 0,
      default_role: '63d87c24b8bebd75ff934267',
    });

    const projId = created.proj_id;

    const allInvites: { id: string; role: 'translator' | 'proofreader' | 'typesetter' }[] = [
      ...invitedTranslators.value.map(m => ({ id: m.id, role: 'translator' as const })),
      ...invitedProofreaders.value.map(m => ({ id: m.id, role: 'proofreader' as const })),
      ...invitedTypesetters.value.map(m => ({ id: m.id, role: 'typesetter' as const })),
    ];

    const assignPromises = allInvites.map(invite =>
      assignMemberToProj({
        proj_id: projId,
        member_id: invite.id,
        is_translator: invite.role === 'translator',
        is_proofreader: invite.role === 'proofreader',
        is_typesetter: invite.role === 'typesetter',
        is_principal: false,
      })
    );

    const settleResults = await Promise.allSettled(assignPromises);
    const assignFailed = settleResults.filter(r => r.status === 'rejected').length;

    if (assignFailed > 0) {
      message.value = `项目创建成功，但有 ${assignFailed} 个成员指派失败`;
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
        <p class="creator-subtitle">填写基础信息并配置权限</p>
      </div>
      <div class="creator-header__right">
        <button type="button" class="creator-close" @click="handleClose" title="关闭">×</button>
      </div>
    </header>

    <div class="creator-body">
      <form class="creator-form" @submit.prevent="handleCreateProject">
        <div class="creator-field">
          <label for="creator-author" class="creator-label">作者</label>
          <input
            id="creator-author"
            v-model="authorName"
            type="text"
            required
            placeholder="请输入作者名称"
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
            placeholder="无须输入序号，其会由服务器自动生成"
            class="creator-input"
          />
          <div v-if="finalTitlePreview" class="creator-title-preview">
            <span class="creator-title-preview__label">最终标题预览：</span>
            <span class="creator-title-preview__text">{{ finalTitlePreview }}</span>
          </div>
        </div>

        <div class="creator-field">
          <label class="creator-label">所属项目集</label>
          <select
            v-model="selectedProjsetId"
            class="creator-input"
            :disabled="!currentProjsets.length"
          >
            <option value="" disabled>请选择项目集</option>
            <option v-for="ps in currentProjsets" :key="ps.projset_id" :value="ps.projset_id">
              {{ ps.projset_name }}
            </option>
          </select>
        </div>

        <div class="creator-field">
          <label for="creator-desc" class="creator-label">描述</label>
          <textarea
            id="creator-desc"
            v-model="projectInfo.description"
            rows="4"
            required
            placeholder="项目描述不超过 4 行"
            class="creator-textarea"
          ></textarea>
        </div>

        <div class="creator-field">
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
      </form>

      <div v-if="message" :class="['creator-message', messageClass]">
        {{ message }}
      </div>

      <div class="creator-actions">
        <button type="submit" class="creator-submit" :disabled="loading">
          {{ loading ? '正在创建...' : '确认创建' }}
        </button>
      </div>

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

      <!-- MemberSelector 悬浮窗（示例） -->
      <div v-if="selectorOpen" class="selector-overlay">
        <div class="selector-panel">
          <header class="selector-header">
            <span class="selector-title">
              {{
                selectorRole === 'translator'
                  ? '选择翻译'
                  : selectorRole === 'proofreader'
                    ? '选择校对'
                    : '选择嵌字'
              }}
            </span>
            <!-- 顶部不再使用关闭叉号，由底部按钮控制 -->
          </header>
          <div class="selector-body">
            <div class="selector-search">
              <input
                v-model="selectorKeyword"
                type="text"
                class="selector-input"
                placeholder="输入成员名称进行搜索"
                @keyup.enter="handleSearchMembers"
              />
              <button
                type="button"
                class="selector-search-btn"
                @click="handleSearchMembers"
                :disabled="selectorLoading"
              >
                {{ selectorLoading ? '搜索中...' : '搜索' }}
              </button>
            </div>
            <ul class="selector-list" v-if="selectorResults.length">
              <li
                v-for="item in selectorResults"
                :key="item.member_id"
                class="selector-item"
                :class="{
                  'selector-item--picked':
                    (selectorRole === 'translator' &&
                      invitedTranslators.some(m => m.id === item.member_id)) ||
                    (selectorRole === 'proofreader' &&
                      invitedProofreaders.some(m => m.id === item.member_id)) ||
                    (selectorRole === 'typesetter' &&
                      invitedTypesetters.some(m => m.id === item.member_id)),
                }"
              >
                <span class="selector-item__name">{{ item.username }}</span>
                <button
                  v-if="
                    (selectorRole === 'translator' &&
                      invitedTranslators.some(m => m.id === item.member_id)) ||
                    (selectorRole === 'proofreader' &&
                      invitedProofreaders.some(m => m.id === item.member_id)) ||
                    (selectorRole === 'typesetter' &&
                      invitedTypesetters.some(m => m.id === item.member_id))
                  "
                  type="button"
                  class="selector-icon-btn selector-icon-btn--remove"
                  @click.stop="handleRemoveMember(item.member_id)"
                  title="移除该成员"
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 16 16"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path
                      d="M4.2 4.2L11.8 11.8M11.8 4.2L4.2 11.8"
                      stroke="currentColor"
                      stroke-width="1.6"
                      stroke-linecap="round"
                    />
                  </svg>
                </button>
                <button
                  v-else
                  type="button"
                  class="selector-icon-btn selector-icon-btn--add"
                  @click.stop="handleSelectMember(item.member_id)"
                  title="加入该成员"
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 16 16"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path
                      d="M8 3V13M3 8H13"
                      stroke="currentColor"
                      stroke-width="1.6"
                      stroke-linecap="round"
                    />
                  </svg>
                </button>
              </li>
            </ul>
            <div v-else class="selector-empty">暂无搜索结果</div>

            <!-- 当前角色已选择成员预览 -->
            <!-- <div
              class="selector-picked"
              v-if="selectorRole === 'translator' && invitedTranslators.length"
            >
              <span class="selector-picked-label">已选翻译：</span>
              <span class="selector-picked-names">{{ invitedTranslators.join('、') }}</span>
            </div>
            <div
              class="selector-picked"
              v-else-if="selectorRole === 'proofreader' && invitedProofreaders.length"
            >
              <span class="selector-picked-label">已选校对：</span>
              <span class="selector-picked-names">{{ invitedProofreaders.join('、') }}</span>
            </div>
            <div
              class="selector-picked"
              v-else-if="selectorRole === 'typesetter' && invitedTypesetters.length"
            >
              <span class="selector-picked-label">已选嵌字：</span>
              <span class="selector-picked-names">{{ invitedTypesetters.join('、') }}</span>
            </div> -->

            <div class="selector-actions">
              <button
                type="button"
                class="selector-action-btn selector-action-btn--cancel"
                @click="handleCancelSelector"
              >
                取消
              </button>
              <button
                type="button"
                class="selector-action-btn selector-action-btn--confirm"
                @click="handleConfirmSelector"
              >
                确认
              </button>
            </div>
          </div>
        </div>
      </div>
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

.creator-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
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
  width: 100%;
  padding: 8px 11px;
  border-radius: 10px;
  border: 1px solid rgba(170, 190, 215, 0.9);
  font-size: 14px;
  box-sizing: border-box;
  resize: none;
  outline: none;
  background: #ffffff;
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.creator-textarea:focus {
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 0 0 1px rgba(118, 184, 255, 0.55);
}

.creator-title-preview {
  margin-top: 4px;
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  padding: 4px 10px;
  border-radius: 999px;
  background: #e6f6ec;
  border: 1px solid rgba(110, 190, 150, 0.7);
  font-size: 12px;
  color: #24563b;
  box-shadow: 0 4px 12px rgba(140, 200, 170, 0.25);
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

.creator-perm-btn {
  flex: 1;
  padding: 8px 10px;
  border-radius: 999px;
  border: 1px solid rgba(170, 190, 215, 0.9);
  background: #ffffff;
  font-size: 13px;
  font-weight: 500;
  color: #23415b;
  cursor: pointer;
  transition:
    background 0.18s ease,
    box-shadow 0.18s ease,
    border-color 0.18s ease,
    transform 0.18s ease;
}

.creator-perm-btn:hover {
  background: #f4f7fb;
  box-shadow: 0 6px 16px rgba(150, 190, 235, 0.25);
  transform: translateY(-1px);
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

.creator-submit {
  min-width: 130px;
  padding: 12px 22px;
  border-radius: 999px;
  border: none;
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #ffffff;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.4px;
  cursor: pointer;
  box-shadow: 0 10px 24px rgba(110, 170, 235, 0.5);
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease,
    opacity 0.18s ease;
}

.creator-submit:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 14px 32px rgba(110, 170, 235, 0.65);
}

.creator-submit:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
}

.creator-message {
  margin-top: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  font-size: 13px;
  text-align: center;
}

.creator-message--success {
  background: #e6f6ec;
  color: #24563b;
  border: 1px solid rgba(110, 190, 150, 0.8);
}

.creator-message--error {
  background: #ffecec;
  color: #7d3434;
  border: 1px solid rgba(220, 140, 140, 0.85);
}

/* 邀请按钮行 */
/* 预邀请成员块 */
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
  padding: 7px 14px;
  border-radius: 999px;
  border: none;
  background: linear-gradient(135deg, #6bb4ff, #4b8fe8);
  color: #ffffff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  box-shadow: 0 6px 16px rgba(110, 170, 235, 0.5);
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease;
}

.creator-invite-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 24px rgba(110, 170, 235, 0.65);
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
</style>
