# 筛选功能架构重构总结

## 问题描述

原有的筛选功能使用了 `v-model` 双向绑定配合 computed setter，导致多个组件持有同一个数组的可变引用。当删除筛选项时，Vue 的响应式系统在同步更新过程中产生冲突，导致以下错误：

- `Cannot set properties of null (setting '__vnode')`
- JSON.parse 错误（undefined）
- 无限循环调用 clearAllOptions

## 根本原因

在 `ProjectFilterBoard.vue` 中使用 computed 属性的 setter 直接修改父组件的状态：

```typescript
// 旧的问题代码
const filterOptions = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

// 直接修改导致同步冲突
filterOptions.value = [];
```

这种模式违反了 Vue 的单向数据流原则，导致状态更新时机不可预测。

## 解决方案

采用事件驱动的单向数据流架构：

### 1. 数据流方向

```
PanelView (状态持有者)
    ↓ props (只读)
ProjectFilterBoard (UI 控制器)
    ↑ events (add-option, remove-option, clear-all)
PanelView (响应事件，更新状态)
    ↓ props (filters)
ProjectList (数据展示)
```

### 2. 核心改动

#### ProjectFilterBoard.vue

**移除：**
- `modelValue` prop
- `update:modelValue` emit
- computed `filterOptions` with setter

**新增：**
- `isSearching` prop：禁用筛选板的标志
- `localFilterOptions` ref：本地预览状态
- 三个新事件：
  - `add-option`：添加筛选项
  - `remove-option`：移除筛选项
  - `clear-all`：清空所有筛选项

**示例代码：**
```typescript
// 本地预览状态
const localFilterOptions = ref<FilterOption[]>([]);

// 添加选项
function addOption(option: FilterOption) {
  localFilterOptions.value.push(option);
  emit('add-option', option); // 通知父组件
}

// 移除选项
function removeOption(option: FilterOption) {
  const index = localFilterOptions.value.findIndex(
    opt => opt.key === option.key && opt.value === option.value
  );
  if (index !== -1) {
    localFilterOptions.value.splice(index, 1);
    emit('remove-option', option); // 通知父组件
  }
}

// 清空选项
function clearAllOptions() {
  localFilterOptions.value = [];
  emit('clear-all'); // 通知父组件
}
```

#### PanelView.vue

**新增：**
- `isSearching` ref：追踪搜索状态
- 三个事件处理函数：
  - `handleAddFilterOption`
  - `handleRemoveFilterOption`
  - `handleClearFilters`

**示例代码：**
```typescript
// 筛选选项数组（状态持有者）
const currentFilterOptions = ref<FilterOption[]>([]);

// 追踪搜索状态
const isSearching = ref(false);

// 处理添加筛选项
function handleAddFilterOption(option: FilterOption) {
  const exists = currentFilterOptions.value.find(
    opt => opt.key === option.key && opt.value === option.value
  );
  if (!exists) {
    currentFilterOptions.value.push(option);
  }
}

// 处理移除筛选项
function handleRemoveFilterOption(option: FilterOption) {
  const index = currentFilterOptions.value.findIndex(
    opt => opt.key === option.key && opt.value === option.value
  );
  if (index !== -1) {
    currentFilterOptions.value.splice(index, 1);
  }
}

// 处理清空筛选项
function handleClearFilters() {
  currentFilterOptions.value = [];
}

// 监控筛选条件变化，更新搜索状态
watch(
  () => currentSearchFilters.value,
  () => {
    isSearching.value = true;
    setTimeout(() => {
      isSearching.value = false;
    }, 500);
  },
  { deep: true }
);
```

**模板更新：**
```vue
<ProjectFilterBoard
  :team-id="activeTeamId ?? undefined"
  :is-searching="isSearching"
  :disabled="viewMode === 'assignments'"
  @add-option="handleAddFilterOption"
  @remove-option="handleRemoveFilterOption"
  @clear-all="handleClearFilters"
  ref="filterBoardRef"
/>
```

### 3. 其他修复

#### ProjectList.vue

修复了 JSON.parse undefined 错误：

```typescript
watch(
  () => props.filters,
  (newFilters, oldFilters) => {
    const newStr = newFilters ? JSON.stringify(newFilters) : undefined;
    const oldStr = oldFilters ? JSON.stringify(oldFilters) : undefined;

    if (newStr !== oldStr) {
      console.log('[ProjectList] Filters changed, resetting to page 1');
      // ...
    }
  },
  { deep: true }
);
```

## 架构优势

1. **单向数据流**：数据只从父组件流向子组件，事件从子组件流向父组件
2. **状态隔离**：每个组件只负责自己的状态，避免共享可变引用
3. **可预测性**：事件触发和状态更新的时序清晰明确
4. **可维护性**：职责分离，易于调试和扩展

## 测试建议

运行开发服务器进行测试：

```bash
make dev
```

测试场景：
1. 添加筛选项（项目名称、状态、时间、成员、项目集）
2. 删除单个筛选项
3. 清空所有筛选项
4. 快速连续操作（验证防抖）
5. 验证搜索期间筛选板是否正确禁用

## 相关文件

- `src/api/model/filterOption.ts`：筛选项类型定义
- `src/components/ProjectFilterBoard.vue`：筛选控制板（事件发射器）
- `src/views/PanelView.vue`：主面板视图（状态持有者）
- `src/components/ProjectList.vue`：项目列表（数据展示）
