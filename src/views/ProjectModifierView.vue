<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useToastStore } from '../stores/toast';
import { useTokenStore } from '../stores/token';
import { assignMemberToProj, updateProjStatus, publishProj } from '../ipc/project';
import { deleteFileCache } from '../ipc/image_cache';
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
  position: 'translator' | 'proofreader' | 'typesetter' | 'redrawer' | 'principal';
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
const currentRedrawers = ref<MemberInfo[]>([]);
// 监修角色仅显示，不允许通过此界面修改（PopRaKo API 不支持 assign principal）
const displayPrincipals = ref<MemberInfo[]>([]);

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

    currentRedrawers.value = props.members
      .filter(m => m.isRedrawer)
      .map(m => ({ id: m.memberId, name: m.username || m.userId || m.memberId }));

    displayPrincipals.value = props.members
      .filter(m => m.isPrincipal)
      .map(m => ({ id: m.memberId, name: m.username || m.userId || m.memberId }));
  }
});

// Member selector state
const selectorOpen = ref(false);
const selectorRole = ref<'translator' | 'proofreader' | 'typesetter' | 'redrawer' | null>(null);
const pickedAll = ref<_ModifierPickedItem[]>([]);

function openSelector(): void {
  console.log('[ProjectModifier] openSelector called', {
    teamId: props.teamId,
    pickedAllCount: pickedAll.value.length,
  });

  selectorRole.value = 'translator';
  // build pickedAll from current lists
  pickedAll.value = [
    ...currentTranslators.value.map(m => ({ ...m, position: 'translator' as const })),
    ...currentProofreaders.value.map(m => ({ ...m, position: 'proofreader' as const })),
    ...currentTypesetters.value.map(m => ({ ...m, position: 'typesetter' as const })),
    ...currentRedrawers.value.map(m => ({ ...m, position: 'redrawer' as const })),
  ];
  selectorOpen.value = true;

  console.log('[ProjectModifier] selector opened', {
    selectorOpen: selectorOpen.value,
    pickedAll: pickedAll.value,
  });
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
  currentRedrawers.value = pickedAll.value
    .filter(p => p.position === 'redrawer')
    .map(p => ({ id: p.id, name: p.name }));

  closeSelector();
}

function onMemberSelectorCancel(): void {
  // MemberSelector 内部已回滚 picked 数组，无需额外处理
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

    // 2. 更新所有成员的角色分配（检测变更并调用 assign API）
    const allCurrentMemberIds = new Set([
      ...currentTranslators.value.map(m => m.id),
      ...currentProofreaders.value.map(m => m.id),
      ...currentTypesetters.value.map(m => m.id),
      ...currentRedrawers.value.map(m => m.id),
    ]);

    // 为每个当前成员调用 assign API 设置其所有角色
    let assignedCount = 0;
    for (const memberId of allCurrentMemberIds) {
      await assignMemberToProj({
        projId: props.projectId,
        memberId,
        isTranslator: currentTranslators.value.some(m => m.id === memberId),
        isProofreader: currentProofreaders.value.some(m => m.id === memberId),
        isTypesetter: currentTypesetters.value.some(m => m.id === memberId),
        isRedrawer: currentRedrawers.value.some(m => m.id === memberId),
      });
      assignedCount++;
    }

    if (assignedCount > 0) {
      toastStore.show(`已更新 ${assignedCount} 名成员的角色分配`);
    }

    // 3. Publish project (if user marked pending publish and not already published)
    if (pendingPublish.value && !props.isPublished) {
      await publishProj({ projId: props.projectId });
      // mark as published locally and clear pending flag
      localIsPublished.value = true;
      pendingPublish.value = false;
      toastStore.show('项目已标记为已发布');
    }

    // 4. 清除项目缓存（图片缓存）以避免一致性问题
    try {
      await deleteFileCache(props.projectId);
      console.log('[ProjectModifier] 项目缓存已清除', props.projectId);
    } catch (cacheErr) {
      console.warn('[ProjectModifier] 清除缓存失败（可忽略）', cacheErr);
    }

    toastStore.show('项目更新成功', 'success');
    emit('close');
  } catch (err) {
    console.error('[ProjectModifier] update failed', err);
    toastStore.show('项目更新失败，请稍后重试', 'error');
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

        <!-- Phase statuses (button-style layout) -->
        <div class="modifier-field">
          <h3 class="modifier-section-title">项目状态</h3>

          <!-- 翻译状态 -->
          <div class="status-row-new">
            <label class="status-label-new">翻译</label>
            <div class="status-buttons">
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      translatingStatus === 0 && (props.translatingStatus ?? 0) === 0,
                  },
                  {
                    'status-btn--modified':
                      translatingStatus === 0 && (props.translatingStatus ?? 0) !== 0,
                  },
                ]"
                @click="translatingStatus = 0"
              >
                未开始
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      translatingStatus === 1 && (props.translatingStatus ?? 0) === 1,
                  },
                  {
                    'status-btn--modified':
                      translatingStatus === 1 && (props.translatingStatus ?? 0) !== 1,
                  },
                ]"
                @click="translatingStatus = 1"
              >
                进行中
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      translatingStatus === 2 && (props.translatingStatus ?? 0) === 2,
                  },
                  {
                    'status-btn--modified':
                      translatingStatus === 2 && (props.translatingStatus ?? 0) !== 2,
                  },
                ]"
                @click="translatingStatus = 2"
              >
                已完成
              </button>
            </div>
          </div>

          <!-- 校对状态 -->
          <div class="status-row-new">
            <label class="status-label-new">校对</label>
            <div class="status-buttons">
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      proofreadingStatus === 0 && (props.proofreadingStatus ?? 0) === 0,
                  },
                  {
                    'status-btn--modified':
                      proofreadingStatus === 0 && (props.proofreadingStatus ?? 0) !== 0,
                  },
                ]"
                @click="proofreadingStatus = 0"
              >
                未开始
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      proofreadingStatus === 1 && (props.proofreadingStatus ?? 0) === 1,
                  },
                  {
                    'status-btn--modified':
                      proofreadingStatus === 1 && (props.proofreadingStatus ?? 0) !== 1,
                  },
                ]"
                @click="proofreadingStatus = 1"
              >
                进行中
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      proofreadingStatus === 2 && (props.proofreadingStatus ?? 0) === 2,
                  },
                  {
                    'status-btn--modified':
                      proofreadingStatus === 2 && (props.proofreadingStatus ?? 0) !== 2,
                  },
                ]"
                @click="proofreadingStatus = 2"
              >
                已完成
              </button>
            </div>
          </div>

          <!-- 嵌字状态 -->
          <div class="status-row-new">
            <label class="status-label-new">嵌字</label>
            <div class="status-buttons">
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      typesettingStatus === 0 && (props.typesettingStatus ?? 0) === 0,
                  },
                  {
                    'status-btn--modified':
                      typesettingStatus === 0 && (props.typesettingStatus ?? 0) !== 0,
                  },
                ]"
                @click="typesettingStatus = 0"
              >
                未开始
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      typesettingStatus === 1 && (props.typesettingStatus ?? 0) === 1,
                  },
                  {
                    'status-btn--modified':
                      typesettingStatus === 1 && (props.typesettingStatus ?? 0) !== 1,
                  },
                ]"
                @click="typesettingStatus = 1"
              >
                进行中
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      typesettingStatus === 2 && (props.typesettingStatus ?? 0) === 2,
                  },
                  {
                    'status-btn--modified':
                      typesettingStatus === 2 && (props.typesettingStatus ?? 0) !== 2,
                  },
                ]"
                @click="typesettingStatus = 2"
              >
                已完成
              </button>
            </div>
          </div>

          <!-- 监修状态 -->
          <div class="status-row-new">
            <label class="status-label-new">监修</label>
            <div class="status-buttons">
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      reviewingStatus === 0 && (props.reviewingStatus ?? 0) === 0,
                  },
                  {
                    'status-btn--modified':
                      reviewingStatus === 0 && (props.reviewingStatus ?? 0) !== 0,
                  },
                ]"
                @click="reviewingStatus = 0"
              >
                未开始
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      reviewingStatus === 1 && (props.reviewingStatus ?? 0) === 1,
                  },
                  {
                    'status-btn--modified':
                      reviewingStatus === 1 && (props.reviewingStatus ?? 0) !== 1,
                  },
                ]"
                @click="reviewingStatus = 1"
              >
                进行中
              </button>
              <button
                type="button"
                :class="[
                  'status-btn',
                  {
                    'status-btn--current':
                      reviewingStatus === 2 && (props.reviewingStatus ?? 0) === 2,
                  },
                  {
                    'status-btn--modified':
                      reviewingStatus === 2 && (props.reviewingStatus ?? 0) !== 2,
                  },
                ]"
                @click="reviewingStatus = 2"
              >
                已完成
              </button>
            </div>
          </div>

          <!-- 发布状态 -->
          <div class="status-row-new">
            <label class="status-label-new">发布</label>
            <div class="status-buttons">
              <button
                type="button"
                :class="['status-btn-publish', { 'status-btn-publish--pending': pendingPublish }]"
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

        <!-- Member management -->
        <div class="modifier-invite-block">
          <div class="modifier-invite-header">
            <h3 class="modifier-section-title">成员管理</h3>
            <button type="button" class="modifier-manage-all-btn" @click="openSelector()">
              管理成员
            </button>
          </div>
          <p class="modifier-section-note">已有成员不可删除；可添加新成员并分配角色。</p>

          <!-- Current members summary -->
          <div class="modifier-member-summary">
            <div class="modifier-member-row">
              <span class="modifier-member-role">翻译:</span>
              <span class="modifier-member-names">
                {{ currentTranslators.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <div class="modifier-member-row">
              <span class="modifier-member-role">校对:</span>
              <span class="modifier-member-names">
                {{ currentProofreaders.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <div class="modifier-member-row">
              <span class="modifier-member-role">嵌字:</span>
              <span class="modifier-member-names">
                {{ currentTypesetters.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <div class="modifier-member-row">
              <span class="modifier-member-role">美工:</span>
              <span class="modifier-member-names">
                {{ currentRedrawers.map(m => m.name).join('、') || '未分配' }}
              </span>
            </div>
            <div class="modifier-member-row modifier-member-row--readonly">
              <span class="modifier-member-role">监修:</span>
              <span class="modifier-member-names">
                {{ displayPrincipals.map(m => m.name).join('、') || '未分配' }}
              </span>
              <span class="modifier-member-note">(仅创建时指定)</span>
            </div>
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
    <MemberSelector
      :show="selectorOpen"
      :team-id="props.teamId"
      :initial-role="selectorRole!"
      :picked="pickedAll"
      @confirm="onMemberSelectorConfirm"
      @cancel="onMemberSelectorCancel"
      @close="closeSelector"
    />
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

.modifier-invite-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.modifier-manage-all-btn {
  padding: 8px 16px;
  border-radius: 10px;
  border: 1px solid rgba(118, 184, 255, 0.35);
  background: #f4f9ff;
  color: #2f5a8f;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.06);
  transition:
    box-shadow 0.18s ease,
    transform 0.18s ease,
    background 0.18s ease;
}

.modifier-manage-all-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 26px rgba(118, 184, 255, 0.12);
  background: #eef6ff;
}

.modifier-member-summary {
  padding: 12px;
  background: #f6f8fc;
  border-radius: 12px;
  border: 1px solid rgba(170, 190, 215, 0.6);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.modifier-member-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
}

.modifier-member-row--readonly {
  opacity: 0.75;
}

.modifier-member-role {
  min-width: 40px;
  font-weight: 600;
  color: #23415b;
}

.modifier-member-names {
  color: #4a5f7a;
  flex: 1;
}

.modifier-member-note {
  margin-left: 6px;
  font-size: 11px;
  color: #7a8b99;
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

/* 新的状态按钮布局 */
.status-row-new {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}

.status-label-new {
  min-width: 60px;
  font-size: 13px;
  font-weight: 600;
  color: #446585;
}

.status-buttons {
  display: flex;
  gap: 8px;
  flex: 1;
}

.status-btn {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid rgba(180, 206, 238, 0.7);
  background: rgba(246, 250, 255, 0.9);
  color: #2b577e;
  font-size: 12px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  transition:
    background 0.12s ease,
    transform 0.12s ease,
    box-shadow 0.12s ease,
    border-color 0.12s ease;
}

.status-btn:hover {
  background: #eaf6ff;
  box-shadow: 0 6px 18px rgba(118, 184, 255, 0.08);
  transform: translateY(-2px);
}

/* 当前状态样式（淡绿色） */
.status-btn--current {
  background: #effaf1;
  border: 1px solid rgba(60, 140, 90, 0.5);
  color: #2f6f45;
  box-shadow: 0 4px 12px rgba(60, 140, 90, 0.12);
}

/* 修改后的状态样式（淡橙黄色） */
.status-btn--modified {
  background: #fff4e6;
  border: 1px solid #ffb84d;
  color: #9a4f00;
  box-shadow: 0 4px 12px rgba(255, 184, 77, 0.12);
}

/* 发布按钮特殊样式 */
.status-btn-publish {
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid rgba(60, 140, 90, 0.5);
  background: #effaf1;
  color: #2f6f45;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 0.12s ease,
    transform 0.12s ease,
    box-shadow 0.12s ease;
}

.status-btn-publish:hover:not(:disabled) {
  background: #e0f5e4;
  box-shadow: 0 6px 18px rgba(60, 140, 90, 0.15);
  transform: translateY(-2px);
}

.status-btn-publish:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.status-btn-publish--pending {
  background: #fff4e6;
  border: 1px solid #ffb84d;
  color: #9a4f00;
  box-shadow: 0 8px 18px rgba(255, 184, 77, 0.12);
}

.status-btn-publish--pending:hover:not(:disabled) {
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
