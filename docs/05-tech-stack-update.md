# XJasper 技术方案更新（2026-03-01）

## 更新内容

基于讨论，对原技术设计进行以下调整：

---

## 1. 前端技术栈调整

### 原方案
- React 18 + React Konva
- Zustand 状态管理

### 调整后
- **Vue 3 + vue-konva**
- **Pinia 状态管理**

### 调整理由

1. **更好的 Vite 集成** — Vite 作者就是 Vue 作者，原生支持最佳
2. **更小的包体积** — Vue 3 运行时约 13KB（gzip），React 约 40KB
3. **Composition API** — 逻辑复用更灵活，代码组织更清晰
4. **更好的 TypeScript 支持** — Vue 3 用 TypeScript 重写
5. **性能优势** — 编译时优化、Proxy 响应式系统

---

## 2. 架构简化

### 原方案：13 个 Rust Crate
```
xjasper-types
xjasper-expression
xjasper-template
xjasper-compiler
xjasper-datasource
xjasper-variables
xjasper-layout
xjasper-renderer-pdf
xjasper-renderer-image
xjasper-renderer-html
xjasper-engine
xjasper-wasm
xjasper-cli
```

### 调整后：7 个 Rust Crate
```
xjasper-core          # types + expression + template + compiler（合并）
xjasper-data          # datasource + variables（合并）
xjasper-layout        # 布局引擎
xjasper-render        # pdf + image + html 渲染器（合并）
xjasper-engine        # 门面
xjasper-wasm          # WASM 绑定
xjasper-cli           # CLI 工具
```

### 调整理由

1. **降低复杂度** — 减少跨 crate 边界的通信开销
2. **更快的编译** — crate 数量减少，编译时间缩短
3. **更容易重构** — 初期模块边界不稳定，合并后调整更灵活
4. **保持清晰** — 核心职责仍然分离（core、data、layout、render）

---

## 3. 开发路线调整

### 原方案：线性开发
```
Phase 1: Rust 核心 (4-6周)
  ↓
Phase 2: WASM 封装 (2-3周)
  ↓
Phase 3: 设计器 UI (6-8周)
  ↓
Phase 4: Tauri 桌面 (3-4周)
  ↓
Phase 5: 金融增强 (4-6周)
```

### 调整后：并行开发
```
Stage 0: 基础设施 (1周)
  - 定义 JSON 模板格式 v0.1
  - 创建示例模板和数据
  ↓
Stage 1: 最小引擎 (2-3周)
  - 简单的 Rust 引擎（title + detail + summary）
  - CLI 工具
  - WASM 封装
  ↓
Stage 2: 并行开发 (6-8周)
  ├─ 分支 A: feature/engine-core
  │   - 完整的 Band 模型
  │   - 分组和聚合
  │   - 分页逻辑
  │
  └─ 分支 B: feature/designer-ui
      - Vue 3 + vue-konva 设计器
      - 基于 Stage 1 的 WASM 引擎
      - 实时预览
  ↓
Stage 3: 集成和优化 (2-3周)
  ↓
Stage 4: Tauri 桌面 (2-3周)
  ↓
Stage 5: 金融增强 (独立分支，可选)
```

### 调整理由

1. **降低风险** — Stage 1 快速验证技术可行性
2. **并行开发** — 引擎和设计器独立推进，互不阻塞
3. **快速迭代** — 设计器可以基于简单引擎先做起来
4. **Worktree 隔离** — 两个独立工作目录，避免分支切换

---

## 4. 表达式引擎待定

### 原方案
- 使用 rhai 作为表达式引擎

### 调整后
- **Stage 1 先用 rhai 快速验证**
- **Stage 2 评估是否自研轻量级解释器**

### 理由

**rhai 的问题：**
- 功能过重（支持函数定义、循环等）
- 报表表达式只需要简单的计算和字段引用
- 安全沙箱的性能开销

**自研的优势：**
- 只支持必要的操作（字段引用、运算、条件、函数调用）
- 更小的包体积
- 更好的性能

**决策点：** Stage 1 用 rhai 快速实现，如果性能或体积成为瓶颈，Stage 2 再考虑自研。

---

## 5. PDF 渲染库待定

### 原方案
- printpdf + lopdf

### 调整后
- **Stage 1 验证 printpdf 是否满足需求**
- **如果不满足，考虑：**
  1. 纯 lopdf（更底层但更灵活）
  2. 参考 Typst 的 PDF 生成实现

### 理由

**printpdf 的限制：**
- 不支持复杂的文本布局（自动换行、对齐）
- 不支持表格（需要手动画线）
- PDF/A 支持不完整

**决策点：** Stage 1 先用 printpdf 实现简单报表，验证是否满足金融报表的复杂需求。

---

## 6. 更新后的技术栈总览

| 层面 | 技术 | 版本 | 说明 |
|------|------|------|------|
| **核心语言** | Rust | 1.75+ | 引擎核心 |
| **数值计算** | rust_decimal | 1.33 | 金融精度 |
| **PDF 生成** | printpdf / lopdf | 0.7 / 0.32 | Stage 1 验证 |
| **字体处理** | rustybuzz + ttf-parser | 0.14 + 0.20 | 文本布局 |
| **表达式** | rhai（待评估） | 1.17 | Stage 1 验证 |
| **图片渲染** | tiny-skia | 0.11 | PNG/JPEG 输出 |
| **国际化** | icu4x | 1.4 | 多币种格式化 |
| **WASM 绑定** | wasm-bindgen | 0.2 | Rust → JS |
| **桌面框架** | Tauri | 2.0 | 跨平台桌面 |
| **前端框架** | **Vue 3** | 3.4+ | UI 组件 |
| **画布引擎** | **vue-konva** | 3.0+ | 可视化编辑 |
| **状态管理** | **Pinia** | 2.1+ | 状态管理 |
| **样式** | Tailwind CSS | 3+ | CSS 框架 |
| **构建工具** | Vite | 5+ | 前端构建 |
| **包管理** | pnpm | 8+ | 依赖管理 |

---

## 7. 关键决策总结

| 决策点 | 原方案 | 调整后 | 理由 |
|--------|--------|--------|------|
| 前端框架 | React | **Vue 3** | Vite 集成更好、包体更小 |
| 状态管理 | Zustand | **Pinia** | Vue 3 官方推荐 |
| Crate 数量 | 13 个 | **7 个** | 降低复杂度、加快编译 |
| 开发模式 | 线性 | **并行** | 引擎和设计器同步开发 |
| 表达式引擎 | rhai | **rhai（待评估）** | Stage 1 验证，可能自研 |
| PDF 渲染 | printpdf | **printpdf（待验证）** | Stage 1 验证是否满足需求 |

---

## 8. 下一步行动

1. **完成 brainstorming 流程** — 确定 Stage 0 的详细设计
2. **编写实现计划** — 调用 writing-plans skill
3. **开始 Stage 0** — 搭建项目骨架、定义 JSON 格式
4. **进入 Stage 1** — 实现最小引擎

---

## 附录：Vue 3 示例代码

### 设计器主界面

```vue
<!-- TemplateEditor.vue -->
<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTemplateStore } from '@/stores/template';
import Canvas from '@/components/Canvas.vue';
import PropertyPanel from '@/components/PropertyPanel.vue';

const store = useTemplateStore();
const template = computed(() => store.template);

const handleSave = () => {
  store.saveTemplate();
};
</script>

<template>
  <div class="flex h-screen">
    <!-- 左侧工具栏 -->
    <div class="w-16 bg-gray-800 flex flex-col items-center py-4 space-y-4">
      <button class="w-10 h-10 bg-gray-700 rounded hover:bg-gray-600">
        <span class="text-white">T</span>
      </button>
      <button class="w-10 h-10 bg-gray-700 rounded hover:bg-gray-600">
        <span class="text-white">□</span>
      </button>
    </div>

    <!-- 中间画布 -->
    <div class="flex-1 bg-gray-100 p-4">
      <Canvas :template="template" />
    </div>

    <!-- 右侧属性面板 -->
    <div class="w-80 bg-white border-l p-4">
      <PropertyPanel :template="template" @update="store.updateTemplate" />
    </div>
  </div>
</template>
```

### Konva 画布

```vue
<!-- Canvas.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Template } from '@/types';

const props = defineProps<{
  template: Template | null;
}>();

const stageConfig = ref({
  width: 800,
  height: 600,
});

const elements = ref<any[]>([]);

watch(() => props.template, (newTemplate) => {
  if (newTemplate) {
    // 将模板元素转换为 Konva 元素
    elements.value = newTemplate.bands.detail.elements.map((el) => ({
      id: el.id,
      x: el.x,
      y: el.y,
      width: el.width,
      height: el.height,
      text: el.expression,
      draggable: true,
    }));
  }
}, { immediate: true });

const handleDragEnd = (id: string, e: any) => {
  const element = elements.value.find((el) => el.id === id);
  if (element) {
    element.x = e.target.x();
    element.y = e.target.y();
    // 更新模板
  }
};
</script>

<template>
  <v-stage :config="stageConfig">
    <v-layer>
      <v-rect
        v-for="el in elements"
        :key="el.id"
        :config="el"
        @dragend="handleDragEnd(el.id, $event)"
      />
    </v-layer>
  </v-stage>
</template>
```

### Pinia Store

```typescript
// stores/template.ts
import { defineStore } from 'pinia';
import type { Template } from '@/types';

export const useTemplateStore = defineStore('template', {
  state: () => ({
    template: null as Template | null,
    selectedElement: null as string | null,
  }),

  getters: {
    hasTemplate: (state) => state.template !== null,
  },

  actions: {
    setTemplate(template: Template) {
      this.template = template;
    },

    updateTemplate(updates: Partial<Template>) {
      if (this.template) {
        this.template = { ...this.template, ...updates };
      }
    },

    saveTemplate() {
      if (this.template) {
        const json = JSON.stringify(this.template, null, 2);
        // 保存到本地或服务器
        console.log('Saving template:', json);
      }
    },

    selectElement(id: string) {
      this.selectedElement = id;
    },
  },
});
```