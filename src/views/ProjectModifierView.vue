<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useToastStore } from '../stores/toast';
import { useTokenStore } from '../stores/token';
import { assignMemberToProj, updateProjStatus, publishProj } from '../ipc/project';
import MemberSelector from '../components/MemberSelector.vue';
import type { ResMember } from '../api/model/member';

// Props from parent (PanelView): existing project details
const props = defineProps<{
  projectId: string;
  projectName: string;
  projectDescription?: string;
  teamId: string;
  projsetName: string | null;
  members?: ResMember[];
  translatingStatus: number | null;
  proofreadingStatus: number | null;
  typesettingStatus: number | null;
  reviewingStatus: number | null;
  isPublished: boolean;
}>();

interface MemberInfo {
  id: string;
  name: string;
}

// 私有类型：本视图使用的表单数据
interface _ModifierProjectInfo {
  name: string;
  description: string;
}

// 私有类型：pickedAll 列表项
interface _ModifierPickedItem {
  id: string;
  name: string;
  position: 'translator' | 'proofreader' | 'typesetter';
}

// 私有类型：状态更新条目
interface _StatusUpdate {
  type: 'translating' | 'proofreading' | 'typesetting' | 'reviewing';
  oldVal: number;
  newVal: number;
}

const emit = defineEmits<{ (e: 'close'): void; (e: 'back'): void }>();

// Form data
const projectInfo = ref<_ModifierProjectInfo>({
  name: props.projectName,
  description: props.projectDescription ?? '',
});

// Phase statuses (0=pending, 1=wip, 2=completed)
const translatingStatus = ref<number>(props.translatingStatus ?? 0);
const proofreadingStatus = ref<number>(props.proofreadingStatus ?? 0);
const typesettingStatus = ref<number>(props.typesettingStatus ?? 0);
const reviewingStatus = ref<number>(props.reviewingStatus ?? 0);

// Publish state (local control; once published, cannot un-publish)
const localIsPublished = ref<boolean>(props.isPublished);
// Pending publish flag: user clicked Publish but hasn't saved yet
const pendingPublish = ref<boolean>(false);

// Loading states
const loading = ref<boolean>(false);
const message = ref<string>('');

const toastStore = useToastStore();
const tokenStore = useTokenStore();

// Member management: start with existing members
const existingMembers = ref<MemberInfo[]>([]);
const currentTranslators = ref<MemberInfo[]>([]);
const currentProofreaders = ref<MemberInfo[]>([]);
const currentTypesetters = ref<MemberInfo[]>([]);

onMounted(() => {
  // Map props.members to local lists by role
  if (props.members && props.members.length > 0) {
    existingMembers.value = props.members.map(m => ({
      id: m.memberId,
      name: m.username || m.userId || m.memberId,
    }));

    currentTranslators.value = props.members
      .filter(m => m.isTranslator)
      .map(m => ({ id: m.memberId, name: m.username || m.userId || m.memberId }));

    currentProofreaders.value = props.members
      .filter(m => m.isProofreader)
      .map(m => ({ id: m.memberId, name: m.username || m.userId || m.memberId }));

    currentTypesetters.value = props.members
      .filter(m => m.isTypesetter)
      .map(m => ({ id: m.memberId, name: m.username || m.userId || m.memberId }));
  }
});

// Member selector state
const selectorOpen = ref(false);
const selectorRole = ref<'translator' | 'proofreader' | 'typesetter' | null>(null);
const pickedAll = ref<_ModifierPickedItem[]>([]);

function openSelector(role: 'translator' | 'proofreader' | 'typesetter'): void {
  selectorRole.value = role;
  // build pickedAll from current lists
  pickedAll.value = [
    ...currentTranslators.value.map(m => ({ ...m, position: 'translator' as const })),
    ...currentProofreaders.value.map(m => ({ ...m, position: 'proofreader' as const })),
    ...currentTypesetters.value.map(m => ({ ...m, position: 'typesetter' as const })),
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
  currentTranslators.value = pickedAll.value
    .filter(p => p.position === 'translator')
    .map(p => ({ id: p.id, name: p.name }));
  currentProofreaders.value = pickedAll.value
    .filter(p => p.position === 'proofreader')
    .map(p => ({ id: p.id, name: p.name }));
  currentTypesetters.value = pickedAll.value
    .filter(p => p.position === 'typesetter')
    .map(p => ({ id: p.id, name: p.name }));

  closeSelector();
}

function onMemberSelectorCancel(): void {
  // restore from current lists
  currentTranslators.value = pickedAll.value
    .filter(p => p.position === 'translator')
    .map(p => ({ id: p.id, name: p.name }));
  currentProofreaders.value = pickedAll.value
    .filter(p => p.position === 'proofreader')
    .map(p => ({ id: p.id, name: p.name }));
  currentTypesetters.value = pickedAll.value
    .filter(p => p.position === 'typesetter')
    .map(p => ({ id: p.id, name: p.name }));

  closeSelector();
}

// Close the modifier view (emit to parent)
function handleClose(): void {
  // When user cancels modification, return to the detail view instead
  // of closing the entire right sidebar. Emit a 'back' event so parent
  // can switch mode to 'detail'.
  emit('back');
}

// Publish click handler (TODO: implement immediate publish behavior if desired)
function handlePublishClick(): void {
  // Toggle pending publish state only (no API call). Actual publish
  // is performed on Save (handleUpdateProject).
  if (props.isPublished) return;
  pendingPublish.value = !pendingPublish.value;
}

// Status text mapping
const statusTextMap: Record<number, string> = {
  0: '未开始',
  1: '进行中',
  2: '已完成',
};

// Submit: update project info, statuses, members, and optionally publish
async function handleUpdateProject(): Promise<void> {
  if (!tokenStore.moetranToken) {
    toastStore.show('缺少 Moetran Token，无法更新项目', 'error');
    return;
  }

  loading.value = true;
  message.value = '';

  try {
    // 1. Update phase statuses (if changed)
    const statusUpdates: _StatusUpdate[] = [
      {
        type: 'translating',
        oldVal: props.translatingStatus ?? 0,
        newVal: translatingStatus.value,
      },
      {
        type: 'proofreading',
        oldVal: props.proofreadingStatus ?? 0,
        newVal: proofreadingStatus.value,
      },
      {
        type: 'typesetting',
        oldVal: props.typesettingStatus ?? 0,
        newVal: typesettingStatus.value,
      },
      {
        type: 'reviewing',
        oldVal: props.reviewingStatus ?? 0,
        newVal: reviewingStatus.value,
      },
    ];

    for (const upd of statusUpdates) {
      if (upd.newVal !== upd.oldVal) {
        await updateProjStatus({
          projId: props.projectId,
          statusType: upd.type,
          newStatus: upd.newVal,
        });
        toastStore.show(`${upd.type} 状态已更新为 ${statusTextMap[upd.newVal]}`);
      }
    }

    // 2. Assign new members (compare with original)
    const newTranslators = currentTranslators.value.filter(
      m => !existingMembers.value.some(em => em.id === m.id)
    );
    const newProofreaders = currentProofreaders.value.filter(
      m => !existingMembers.value.some(em => em.id === m.id)
    );
    const newTypesetters = currentTypesetters.value.filter(
      m => !existingMembers.value.some(em => em.id === m.id)
    );

    const allNewMembers = new Set([
      ...newTranslators.map(m => m.id),
      ...newProofreaders.map(m => m.id),
      ...newTypesetters.map(m => m.id),
    ]);

    for (const memberId of allNewMembers) {
      await assignMemberToProj({
        projId: props.projectId,
        memberId,
        isTranslator: newTranslators.some(m => m.id === memberId),
        isProofreader: newProofreaders.some(m => m.id === memberId),
        isTypesetter: newTypesetters.some(m => m.id === memberId),
        isPrincipal: false,
      });
    }

    if (allNewMembers.size > 0) {
      toastStore.show(`已添加 ${allNewMembers.size} 名新成员`);
    }

    // 3. Publish project (if user marked pending publish and not already published)
    if (pendingPublish.value && !props.isPublished) {
      await publishProj({ projId: props.projectId });
      // mark as published locally and clear pending flag
      localIsPublished.value = true;
      pendingPublish.value = false;
      toastStore.show('项目已标记为已发布');
    }

    toastStore.show('项目更新成功', 'success');
    emit('close');
  } catch (err) {
    console.error('[ProjectModifier] update failed', err);
    toastStore.show('项目更新失败: ' + (err?.toString?.() ?? String(err)), 'error');
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <section class="modifier-root">
    <header class="modifier-header">
      <div class="modifier-header__left">
        <h1 class="modifier-title">修改项目信息</h1>
        <p class="modifier-subtitle">项目: {{ projectInfo.name }}</p>
      </div>
      <div class="modifier-header__right">
        <button type="button" class="modifier-close" @click="handleClose" title="关闭">×</button>
      </div>
    </header>

    <div class="modifier-body">
      <form class="modifier-form" @submit.prevent="handleUpdateProject">
        <!-- Project name (display only, not editable in this simplified version) -->
        <div class="modifier-field">
          <label class="modifier-label">项目名称</label>
          <input
            v-model="projectInfo.name"
            type="text"
            class="modifier-input"
            placeholder="项目名称"
            disabled
          />
        </div>

        <!-- Project set (display only, cannot be changed) -->
        <div class="modifier-field">
          <label class="modifier-label">所属项目集</label>
          <input :value="projsetName ?? '未知'" type="text" class="modifier-input" disabled />
        </div>

        <!-- Description -->
        <!-- <div class="modifier-field">
          <label class="modifier-label">项目描述</label>
          <textarea
            v-model="projectInfo.description"
            class="modifier-textarea"
            placeholder="项目描述"
            rows="3"
            disabled
          ></textarea>
        </div> -->

        <!-- Phase statuses (compact row) -->
        <div class="modifier-field modifier-status-row">
          <div class="status-row">
            <div class="status-item">
              <label class="modifier-label">翻译</label>
              <select v-model.number="translatingStatus" class="modifier-select status-select">
                <option :value="0">未开始</option>
                <option :value="1">进行中</option>
                <option :value="2">已完成</option>
              </select>
            </div>

            <div class="status-item">
              <label class="modifier-label">校对</label>
              <select v-model.number="proofreadingStatus" class="modifier-select status-select">
                <option :value="0">未开始</option>
                <option :value="1">进行中</option>
                <option :value="2">已完成</option>
              </select>
            </div>

            <div class="status-item">
              <label class="modifier-label">嵌字</label>
              <select v-model.number="typesettingStatus" class="modifier-select status-select">
                <option :value="0">未开始</option>
                <option :value="1">进行中</option>
                <option :value="2">已完成</option>
              </select>
            </div>

            <div class="status-item">
              <label class="modifier-label">监修</label>
              <select v-model.number="reviewingStatus" class="modifier-select status-select">
                <option :value="0">未开始</option>
                <option :value="1">进行中</option>
                <option :value="2">已完成</option>
              </select>
            </div>

            <div class="status-item publish-item">
              <label class="modifier-label">发布</label>
              <div class="modifier-publish-control">
                <button
                  type="button"
                  :class="['modifier-publish-btn', { pending: pendingPublish }]"
                  :disabled="props.isPublished"
                  @click="handlePublishClick"
                >
                  {{ props.isPublished ? '已发布' : pendingPublish ? '待发布' : '发布' }}
                </button>
                <span v-if="props.isPublished" class="modifier-publish-note"
                  >（已发布，不可更改）</span
                >
              </div>
            </div>
          </div>
        </div>

        <!-- Member management -->
        <div class="modifier-invite-block">
          <h3 class="modifier-section-title">成员管理</h3>
          <p class="modifier-section-note">已有成员不可删除；可添加新成员并分配角色。</p>

          <!-- Translators -->
          <div class="modifier-invite-item">
            <div class="modifier-invite-text">
              翻译:
              <span class="modifier-invite-names">
                {{ currentTranslators.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <button type="button" class="modifier-invite-btn" @click="openSelector('translator')">
              管理
            </button>
          </div>

          <!-- Proofreaders -->
          <div class="modifier-invite-item">
            <div class="modifier-invite-text">
              校对:
              <span class="modifier-invite-names">
                {{ currentProofreaders.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <button type="button" class="modifier-invite-btn" @click="openSelector('proofreader')">
              管理
            </button>
          </div>

          <!-- Typesetters -->
          <div class="modifier-invite-item">
            <div class="modifier-invite-text">
              嵌字:
              <span class="modifier-invite-names">
                {{ currentTypesetters.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <button type="button" class="modifier-invite-btn" @click="openSelector('typesetter')">
              管理
            </button>
          </div>
        </div>
      </form>
    </div>

    <footer class="modifier-footer">
      <button type="button" class="modifier-cancel" @click="handleClose" :disabled="loading">
        取消
      </button>

      <button
        type="button"
        class="modifier-submit"
        @click="handleUpdateProject"
        :disabled="loading"
      >
        {{ loading ? '更新中...' : '保存修改' }}
      </button>
    </footer>

    <!-- Member selector modal -->
    <div v-if="selectorOpen" class="selector-overlay" @click.self="closeSelector">
      <div class="selector-panel">
        <MemberSelector
          :show="selectorOpen"
          :team-id="teamId"
          :role="selectorRole!"
          v-model:picked="pickedAll"
          @confirm="onMemberSelectorConfirm"
          @cancel="onMemberSelectorCancel"
        />
      </div>
    </div>
  </section>
</template>

<style scoped>
.modifier-root {
  display: flex;
  flex-direction: column;
  padding: 24px 28px 32px;
  box-sizing: border-box;
  height: 100%;
  background: linear-gradient(180deg, #f5f9ff 0%, #ffffff 65%);
  font-family: 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  color: #1f2e43;
}

.modifier-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
}

.modifier-header__left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.modifier-title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.4px;
}

.modifier-subtitle {
  margin: 0;
  font-size: 13px;
  color: #4a5f7a;
}

.modifier-header__right {
  display: flex;
  align-items: center;
}

.modifier-close {
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
    box-shadow 0.2s ease,
    transform 0.2s ease;
}

.modifier-close:hover {
  box-shadow: 0 8px 20px rgba(140, 180, 230, 0.25);
  transform: translateY(-2px);
}

.modifier-body {
  margin-top: 22px;
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.modifier-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modifier-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.modifier-label {
  font-size: 14px;
  font-weight: 600;
  color: #23415b;
}

.modifier-input,
.modifier-textarea,
.modifier-select {
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

.modifier-input:focus,
.modifier-textarea:focus,
.modifier-select:focus {
  border-color: rgba(118, 184, 255, 0.9);
  box-shadow: 0 0 0 1px rgba(118, 184, 255, 0.55);
}

.modifier-input:disabled,
.modifier-textarea:disabled {
  background: #f6f8fc;
  color: #6b7c91;
  cursor: not-allowed;
}

.modifier-select {
  cursor: pointer;
}

.modifier-publish-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modifier-publish-label {
  font-size: 14px;
  color: #23415b;
  cursor: pointer;
}

.modifier-publish-note {
  font-size: 12px;
  color: #7a8b99;
  margin-left: 6px;
}

.modifier-section-title {
  margin: 14px 0 6px;
  font-size: 16px;
  font-weight: 600;
  color: #23415b;
}

.modifier-section-note {
  margin: 0 0 10px;
  font-size: 12px;
  color: #6b7c91;
}

.modifier-invite-block {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.modifier-invite-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 12px;
  border-radius: 12px;
  background: #f6f8fc;
  border: 1px solid rgba(170, 190, 215, 0.9);
}

.modifier-invite-text {
  font-size: 13px;
  color: #23415b;
}

.modifier-invite-names {
  margin-left: 4px;
  color: #4a5f7a;
}

.modifier-invite-btn {
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
    box-shadow 0.18s ease,
    transform 0.18s ease,
    background 0.18s ease;
}

.modifier-invite-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 20px rgba(118, 184, 255, 0.08);
  background: #eef6ff;
}

.modifier-footer {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
  padding: 12px 28px 18px;
  border-top: 1px solid rgba(170, 190, 215, 0.6);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0), rgba(250, 250, 250, 0.03));
}

.modifier-submit {
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
    box-shadow 0.18s ease,
    transform 0.18s ease,
    background 0.18s ease;
}

.modifier-submit:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.12);
  background: #eef6ff;
}

.modifier-submit:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  box-shadow: none;
}

/* Compact status row */
.status-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-wrap: wrap;
}

.status-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 160px;
}

.status-select {
  min-width: 140px;
}

.publish-item {
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.modifier-publish-btn {
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid rgba(60, 140, 90, 0.12);
  background: #effaf1;
  color: #2f6f45;
  font-weight: 600;
  cursor: pointer;
}

.modifier-publish-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Pending publish visual style */
.modifier-publish-btn.pending {
  background: #fff4e6; /* warm/pending background */
  border: 1px solid #ffb84d; /* stronger orange border */
  color: #9a4f00; /* darker orange text */
  box-shadow: 0 8px 18px rgba(255, 184, 77, 0.12);
}

.modifier-publish-btn.pending:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 12px 30px rgba(255, 184, 77, 0.16);
}

.modifier-cancel {
  margin-right: 12px;
  padding: 10px 16px;
  border-radius: 10px;
  border: 1px solid rgba(180, 180, 180, 0.18);
  background: #ffffff;
  color: #3b4b5a;
  font-size: 14px;
  cursor: pointer;
}

.modifier-cancel:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Member selector overlay */
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

@media (max-width: 960px) {
  .modifier-root {
    padding: 16px 16px 24px;
  }
}
</style>
