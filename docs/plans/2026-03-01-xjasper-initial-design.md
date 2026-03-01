# XJasper 初始设计方案

**日期：** 2026-03-01
**状态：** 已批准
**作者：** Claude Opus 4.6

---

## 1. 概述

XJasper 是一个现代化的报表生成引擎，灵感来自 JasperReports，使用 Rust 构建核心引擎，Vue 3 构建可视化设计器。

### 1.1 核心目标

- **高性能：** Rust 引擎，处理大数据量（10,000+ 行）
- **跨平台：** WASM 支持浏览器运行，Tauri 支持桌面应用
- **金融级精度：** 使用 Decimal128，无浮点误差
- **可视化设计：** Vue 3 + vue-konva 拖拽式设计器
- **实时预览：** 设计器中实时预览 PDF 输出

### 1.2 技术栈

**后端核心（Rust）：**
- 语言：Rust 1.75+
- 数值计算：rust_decimal（Decimal128）
- PDF 生成：printpdf / lopdf
- 字体处理：rustybuzz + ttf-parser
- 表达式引擎：rhai
- 图片渲染：tiny-skia
- 国际化：icu4x
- WASM 绑定：wasm-bindgen

**前端（Vue 3）：**
- 框架：Vue 3 + TypeScript
- 构建工具：Vite 5
- 画布引擎：vue-konva
- 状态管理：Pinia
- 样式：Tailwind CSS
- 包管理：pnpm

**桌面应用：**
- 框架：Tauri 2

---

## 2. 架构设计

### 2.1 四阶段管线

```
JSON 模板 → 编译 → 填充（数据+布局） → 导出（PDF/图片）
```

**阶段说明：**

1. **编译阶段：** 解析 JSON 模板，验证格式，编译表达式
2. **填充阶段：** 加载数据源，执行表达式，计算变量聚合
3. **布局阶段：** 展开 Band，处理分组和分页，计算元素位置
4. **导出阶段：** 渲染为 PDF、图片或 HTML

### 2.2 Crate 结构（7 个模块）

```
xjasper/
├── crates/
│   ├── xjasper-core/       # 核心模块（types + expression + template + compiler）
│   ├── xjasper-data/       # 数据模块（datasource + variables）
│   ├── xjasper-layout/     # 布局引擎
│   ├── xjasper-render/     # 渲染器（pdf + image + html）
│   ├── xjasper-engine/     # 门面模块
│   ├── xjasper-wasm/       # WASM 绑定
│   └── xjasper-cli/        # CLI 工具
├── apps/
│   └── web/                # Vue 3 设计器
├── examples/               # 示例模板和数据
└── docs/                   # 文档
```

### 2.3 模块职责

#### xjasper-core（核心模块）

**职责：** 类型定义、表达式引擎、模板解析、编译器

**主要类型：**
```rust
pub struct Template {
    pub name: String,
    pub version: String,
    pub page: PageConfig,
    pub fields: Vec<Field>,
    pub variables: Vec<Variable>,
    pub bands: BandMap,
}

pub struct Band {
    pub height: u32,
    pub elements: Vec<Element>,
}

pub enum Element {
    StaticText(StaticText),
    TextField(TextField),
    Image(Image),
    Line(Line),
    Rectangle(Rectangle),
}
```

**表达式语法：**
- `$F{fieldName}` — 字段引用
- `$V{variableName}` — 变量引用
- `$P{parameterName}` — 参数引用
- `$R{key}` — 国际化资源

#### xjasper-data（数据模块）

**职责：** 数据源抽象、变量聚合

**数据源接口：**
```rust
pub trait DataSource {
    fn next(&mut self) -> Result<bool>;
    fn get_field(&self, name: &str) -> Result<Value>;
    fn reset(&mut self) -> Result<()>;
}
```

**变量聚合类型：**
- Sum、Count、DistinctCount
- Average、Min、Max
- StdDev、Variance、First

#### xjasper-layout（布局引擎）

**职责：** Band 展开、分组、分页、元素定位

**核心算法：**
1. 遍历数据源
2. 根据分组表达式触发 groupHeader/groupFooter
3. 展开 detail Band
4. 计算页面高度，触发分页
5. 输出 FilledDocument（包含绝对定位的元素）

#### xjasper-render（渲染器）

**职责：** PDF、图片、HTML 渲染

**共享逻辑：**
- 字体加载和管理
- 文本测量和布局
- 颜色处理
- 坐标转换

---

## 3. Band 模型

### 3.1 完整 Band 列表

| Band | 触发时机 | 用途 |
|------|---------|------|
| background | 每页 | 背景（水印、背景图） |
| title | 报表开始 | 标题页 |
| pageHeader | 每页顶部 | 页眉 |
| columnHeader | 每列顶部 | 列标题 |
| groupHeader | 分组开始 | 分组标题 |
| detail | 每行数据 | 明细行 |
| groupFooter | 分组结束 | 分组小计 |
| columnFooter | 每列底部 | 列脚注 |
| pageFooter | 每页底部 | 页脚 |
| lastPageFooter | 最后一页底部 | 最后页脚 |
| summary | 报表结束 | 汇总 |
| noData | 无数据时 | 空数据提示 |

### 3.2 Band 执行顺序

```
开始报表
  ↓
title（一次）
  ↓
[对于每一页]
  ↓
  background
  ↓
  pageHeader
  ↓
  columnHeader
  ↓
  [对于每个分组]
    ↓
    groupHeader
    ↓
    [对于每行数据]
      ↓
      detail
    ↓
    groupFooter
  ↓
  columnFooter
  ↓
  pageFooter（或 lastPageFooter）
  ↓
summary（一次）
  ↓
结束报表
```

---

## 4. JSON 模板格式

### 4.1 最小版本（v0.1）

```json
{
  "name": "simple-invoice",
  "version": "0.1",
  "page": {
    "width": 595,
    "height": 842,
    "margins": [40, 40, 40, 40]
  },
  "fields": [
    { "name": "customerName", "type": "string" },
    { "name": "amount", "type": "decimal" }
  ],
  "variables": [
    {
      "name": "total",
      "type": "decimal",
      "calculation": "Sum",
      "expression": "$F{amount}"
    }
  ],
  "bands": {
    "title": {
      "height": 60,
      "elements": [
        {
          "type": "staticText",
          "x": 0,
          "y": 10,
          "width": 515,
          "height": 40,
          "text": "Invoice",
          "style": {
            "fontSize": 24,
            "fontWeight": "bold"
          }
        }
      ]
    },
    "detail": {
      "height": 20,
      "elements": [
        {
          "type": "textField",
          "x": 0,
          "y": 0,
          "width": 300,
          "height": 20,
          "expression": "$F{customerName}"
        },
        {
          "type": "textField",
          "x": 300,
          "y": 0,
          "width": 215,
          "height": 20,
          "expression": "$F{amount}",
          "style": {
            "align": "right"
          }
        }
      ]
    },
    "summary": {
      "height": 30,
      "elements": [
        {
          "type": "staticText",
          "x": 200,
          "y": 5,
          "width": 100,
          "height": 20,
          "text": "Total:"
        },
        {
          "type": "textField",
          "x": 300,
          "y": 5,
          "width": 215,
          "height": 20,
          "expression": "$V{total}",
          "style": {
            "align": "right",
            "fontWeight": "bold"
          }
        }
      ]
    }
  }
}
```

### 4.2 完整版本（v0.2+）

**新增功能：**
- 分组定义
- 更多元素类型（image、line、rectangle）
- 复杂表达式
- 条件格式
- 子报表

---

## 5. 开发路线

### 5.1 Stage 0：基础设施（1 周）

**任务：**
- ✅ 初始化 Git 仓库
- ✅ 推送到 GitHub
- ⏳ 搭建 Cargo workspace
- ⏳ 定义 JSON 模板格式 v0.1
- ⏳ 创建示例模板和数据
- ⏳ 编写 JSON Schema

**交付物：**
- Git 仓库：https://github.com/xuweizhengo/xjasper
- Cargo workspace 结构
- `examples/simple-invoice.json`
- `examples/data.json`
- `schema/template-v0.1.json`

### 5.2 Stage 1：最小引擎（2-3 周）

**功能范围：**
- 只支持 3 个 Band：title、detail、summary
- 只支持 2 种元素：staticText、textField
- 只支持简单表达式：`$F{field}`
- 只支持 1 个聚合：Sum
- 单页输出（不分页）
- PDF 渲染

**实现顺序：**
1. xjasper-core（3 天）
2. xjasper-data（2 天）
3. xjasper-layout（4 天）
4. xjasper-render（3 天）
5. xjasper-engine（1 天）
6. xjasper-cli（1 天）
7. xjasper-wasm（2 天）

**验收标准：**
```bash
cargo run --bin xjasper-cli -- \
  --template examples/simple-invoice.json \
  --data examples/data.json \
  --output output.pdf
```

### 5.3 Stage 2：并行开发（6-8 周）

#### 分支 A：feature/engine-core（引擎增强）

**工作目录：** `~/projects/xjasper-engine`

**功能清单：**
1. 完整 Band 模型（1 周）
2. 分组机制（1.5 周）
3. 变量聚合完整实现（1 周）
4. 分页逻辑（1.5 周）
5. 更多元素类型（1 周）
6. 复杂表达式（1 周）
7. 多数据源（1 周）

#### 分支 B：feature/designer-ui（设计器开发）

**工作目录：** `~/projects/xjasper-designer`

**功能清单：**
1. 项目搭建（2 天）
2. 画布编辑器（2 周）
3. 属性面板（1 周）
4. Band 管理（1 周）
5. 数据配置（1 周）
6. 实时预览（1 周）
7. 模板管理（3 天）

#### 协作方式

**JSON 模板格式是契约：**
```
Stage 0 定义格式 v0.1
    ↓
Stage 1 实现引擎（支持 v0.1）
    ↓
分支 A 扩展格式 → v0.2（添加分组、分页等）
    ↓
分支 B 升级 WASM 依赖 → 支持 v0.2
```

**版本管理：**
- `@xjasper/core@0.1.0` — Stage 1 最小引擎
- `@xjasper/core@0.2.0` — 添加分组和分页
- `@xjasper/core@0.3.0` — 添加复杂表达式

**定期同步：**
- 每周五合并到 main
- 解决冲突
- 集成测试

### 5.4 Stage 3：集成和优化（2-3 周）

**任务：**
1. 合并分支（3 天）
2. 端到端测试（1 周）
3. 性能优化（1 周）
4. 文档完善（2 天）

**验收标准：**
- 10,000 行数据生成 PDF < 5 秒
- WASM 包体积 < 2MB（gzip 后 < 500KB）
- 设计器流畅运行（60fps）
- 端到端测试覆盖率 > 90%

### 5.5 Stage 4：Tauri 桌面应用（2-3 周）

**任务：**
1. Tauri 集成（3 天）
2. 原生功能（1 周）
3. 打包和分发（3 天）
4. 性能优化（3 天）

**交付物：**
- 跨平台桌面应用
- 安装包（< 50MB）
- 用户手册

### 5.6 Stage 5：金融增强（独立分支，按需开发）

**功能清单：**
1. 交叉表（Crosstab）
2. 子报表（Subreport）
3. PDF/A 合规
4. 水印和签章
5. 高级格式化

---

## 6. 技术风险和缓解措施

### 6.1 风险 1：PDF 生成库不满足需求

**风险描述：**
- printpdf 功能有限
- lopdf 太底层

**缓解措施：**
1. Stage 1 验证 printpdf
2. 备选方案：
   - lopdf + 自己实现
   - wkhtmltopdf
   - Chromium Headless

**决策点：** Stage 1 结束时（2-3 周后）

### 6.2 风险 2：WASM 性能不足

**风险描述：**
- 大数据量时性能可能不如原生
- 浏览器内存限制

**缓解措施：**
1. 分页处理
2. Web Worker
3. 流式渲染
4. 降级方案（提示使用桌面版）

**性能目标：**
- 1,000 行数据：< 1 秒
- 10,000 行数据：< 10 秒

### 6.3 风险 3：并行开发冲突

**风险描述：**
- 引擎和设计器同时修改 JSON 格式
- 合并时冲突严重

**缓解措施：**
1. 格式版本化
2. 向后兼容
3. 定期同步（每周）
4. 契约测试（JSON Schema）

### 6.4 风险 4：字体处理复杂

**风险描述：**
- 中文字体文件大（10-20MB）
- 字体回退逻辑复杂
- 不同平台字体路径不同

**缓解措施：**
1. 字体子集化
2. 默认字体（Noto Sans CJK）
3. 字体配置
4. Web Fonts（WASM 版本）

---

## 7. 项目里程碑

```
Week 1-2:   Stage 0 基础设施
Week 3-5:   Stage 1 最小引擎
Week 6:     验证和调整
Week 7-14:  Stage 2 并行开发
Week 15-17: Stage 3 集成和优化
Week 18-20: Stage 4 Tauri 桌面应用
Week 21+:   Stage 5 金融增强（按需）
```

**总时间：** 约 5 个月（20 周）

**关键里程碑：**
- ✅ Week 2：JSON 格式定义完成
- ✅ Week 5：最小引擎可用（CLI + WASM）
- ✅ Week 6：技术栈验证通过
- ✅ Week 14：引擎和设计器功能完整
- ✅ Week 17：集成测试通过
- ✅ Week 20：桌面应用发布

---

## 8. 下一步行动

1. ✅ 初始化 Git 仓库
2. ✅ 推送到 GitHub
3. ⏳ 搭建 Cargo workspace
4. ⏳ 定义 JSON 模板格式 v0.1
5. ⏳ 创建示例模板和数据
6. ⏳ 开始 Stage 1 开发

---

**批准日期：** 2026-03-01
**批准人：** 用户确认
