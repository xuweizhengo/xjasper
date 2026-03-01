# XJasper 技术设计文档

## 1. 项目概述

### 1.1 项目定位

XJasper 是一个现代化的、Rust 驱动的、面向金融场景的像素级精确报表引擎，包含：
- 核心模板引擎（Rust）
- 高性能渲染引擎（PDF/图片导出）
- 可视化设计器（Web + 桌面）

### 1.2 核心目标

1. **金融级数值精度** — 使用 Decimal128，避免浮点误差
2. **像素级精确布局** — 绝对定位，满足监管报表格式要求
3. **高性能渲染** — Rust 核心，大数据量流式处理
4. **跨平台复用** — 一套代码，WASM 浏览器端 + Tauri 桌面端
5. **现代开发体验** — JSON 模板、Git 友好、可视化设计器

### 1.3 技术栈选型

| 层面 | 技术选型 | 理由 |
|------|---------|------|
| 核心引擎 | Rust | 性能、内存安全、零 GC、WASM 支持 |
| 数值计算 | rust_decimal | 128bit 定点数，金融级精度 |
| PDF 生成 | printpdf + lopdf | 纯 Rust，支持 PDF/A |
| 字体处理 | rustybuzz + ttf-parser | Typst 同款，HarfBuzz Rust 移植 |
| 图片渲染 | tiny-skia | Skia 子集纯 Rust 实现，可编译 WASM |
| 表达式引擎 | rhai | Rust 嵌入式脚本，安全沙箱，类 JS 语法 |
| WASM 绑定 | wasm-bindgen + wasm-pack | Rust → WASM 标准工具链 |
| 桌面框架 | Tauri 2 | Rust 后端 + Web 前端，包体小 |
| 设计器 UI | Vue 3 + vue-konva | 组合式 API 灵活，Vite 原生支持好 |
| 状态管理 | Pinia | Vue 3 官方推荐的状态管理 |
| 数据格式化 | icu4x | Unicode 官方 Rust 实现，多币种/多语言 |

---

## 2. 架构设计

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Designer UI (Vue 3)                       │
│              Canvas Editor + Property Panels                 │
├──────────────────────┬──────────────────────────────────────┤
│   Tauri 2 Bridge     │         WASM Bridge                   │
│   (Desktop Native)   │      (Browser Runtime)                │
├──────────────────────┴──────────────────────────────────────┤
│                  Rust Core Engine                            │
│  ┌────────────┬────────────┬────────────┬────────────┐      │
│  │  Template  │ Expression │  Layout    │  Renderer  │      │
│  │  Parser    │  Engine    │  Engine    │  (PDF/IMG) │      │
│  └────────────┴────────────┴────────────┴────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 四阶段处理管线

借鉴 JasperReports 的设计，XJasper 采用四阶段管线：

```
JSON Template  →  Parsed Template  →  Compiled Template  →  Filled Document  →  Output
   (.json)         (in-memory)          (.xjc cache)         (pages)           (PDF/PNG)
   [Parse]           [Compile]             [Fill]              [Export]
```

| 阶段 | 输入 | 输出 | 核心模块 |
|------|------|------|----------|
| Parse | JSON 模板 | ParsedTemplate | xjasper-template |
| Compile | ParsedTemplate | CompiledTemplate | xjasper-compiler |
| Fill | CompiledTemplate + 数据 | FilledDocument | xjasper-layout |
| Export | FilledDocument | PDF / PNG / HTML | xjasper-renderer-* |

**关键设计点：**
- Compile 阶段预编译表达式 AST，生成可缓存的 .xjc 文件
- Fill 阶段是纯数据处理，可并行处理多个数据集
- Export 阶段完全解耦，同一 FilledDocument 可导出多种格式

---

## 3. 模块拆分

### 3.1 Cargo Workspace 结构

```
xjasper/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── xjasper-types/            # 共享类型定义
│   ├── xjasper-expression/       # 表达式引擎
│   ├── xjasper-template/         # 模板解析
│   ├── xjasper-compiler/         # 模板编译
│   ├── xjasper-datasource/       # 数据源抽象
│   ├── xjasper-variables/        # 变量聚合引擎
│   ├── xjasper-layout/           # 布局引擎
│   ├── xjasper-renderer-pdf/     # PDF 渲染
│   ├── xjasper-renderer-image/   # 图片渲染
│   ├── xjasper-renderer-html/    # HTML 渲染（预览）
│   ├── xjasper-engine/           # 门面模块
│   ├── xjasper-wasm/             # WASM 绑定
│   └── xjasper-cli/              # CLI 工具
├── apps/
│   ├── desktop/                  # Tauri 2 桌面应用
│   │   ├── src-tauri/            # Rust 后端
│   │   └── src/                  # React 前端
│   └── web/                      # 纯 Web 版
└── packages/
    └── designer-ui/              # 设计器 UI 组件（共享）
```

### 3.2 核心 Crate 职责

#### xjasper-types

零依赖的类型定义 crate，所有模块共享。

**核心类型：**
```rust
pub struct Template {
    pub name: String,
    pub page: PageConfig,
    pub parameters: Vec<Parameter>,
    pub fields: Vec<Field>,
    pub variables: Vec<Variable>,
    pub groups: Vec<Group>,
    pub bands: Bands,
    pub styles: Vec<Style>,
}

pub struct Bands {
    pub background: Option<Band>,
    pub title: Option<Band>,
    pub page_header: Option<Band>,
    pub column_header: Option<Band>,
    pub group_headers: Vec<Band>,
    pub detail: Band,
    pub group_footers: Vec<Band>,
    pub column_footer: Option<Band>,
    pub page_footer: Option<Band>,
    pub last_page_footer: Option<Band>,
    pub summary: Option<Band>,
    pub no_data: Option<Band>,
}

pub enum Element {
    StaticText(StaticText),
    TextField(TextField),
    Image(Image),
    Line(Line),
    Rectangle(Rectangle),
    Ellipse(Ellipse),
    Frame(Frame),
    Subreport(Subreport),
}

pub enum BandType {
    Background,
    Title,
    PageHeader,
    ColumnHeader,
    GroupHeader,
    Detail,
    GroupFooter,
    ColumnFooter,
    PageFooter,
    LastPageFooter,
    Summary,
    NoData,
}
```

#### xjasper-expression

表达式解析和求值引擎，支持 `$F{field}` `$V{variable}` `$P{parameter}` 语法。

**核心功能：**
- 表达式词法分析和语法解析
- AST 构建和优化
- 安全沙箱求值（基于 rhai）
- 内置函数库（日期格式化、数字格式化、字符串处理）

**依赖：** xjasper-types, rhai, rust_decimal

#### xjasper-template

JSON 模板解析和校验。

**核心功能：**
- JSON 反序列化为 Template 结构
- JSON Schema 校验
- 模板版本兼容和迁移
- 模板合并（继承、引用）

**依赖：** xjasper-types, serde, serde_json, jsonschema

#### xjasper-compiler

模板编译器，生成优化的编译产物。

**核心功能：**
- 表达式预编译（生成 AST，避免运行时解析）
- 模板一致性校验（字段引用、变量依赖检查）
- 生成可缓存的 .xjc 文件（序列化的 CompiledTemplate）
- 编译优化（常量折叠、死代码消除）

**依赖：** xjasper-types, xjasper-expression, xjasper-template

#### xjasper-datasource

数据源抽象层，统一的数据访问接口。

**核心 trait：**
```rust
pub trait DataSource {
    fn next(&mut self) -> Result<bool>;
    fn get_field(&self, field: &str) -> Result<Value>;
    fn reset(&mut self) -> Result<()>;
}

pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Decimal(Decimal),
    String(String),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Bytes(Vec<u8>),
}
```

**内置实现：**
- JsonDataSource（Vec<Map>）
- CsvDataSource（csv crate）
- EmptyDataSource（测试用）
- 未来扩展：SqlDataSource（sqlx）

**依赖：** xjasper-types, rust_decimal, chrono

#### xjasper-variables

变量聚合引擎，实现 Sum/Count/Avg 等计算。

**核心功能：**
- 变量状态管理（当前值、累加器）
- 聚合计算（Sum、Count、DistinctCount、Average、Min、Max、StdDev、Variance、First）
- 重置逻辑（Report/Page/Column/Group 级别）
- 增量更新（每条记录触发）

**依赖：** xjasper-types, xjasper-expression, rust_decimal

#### xjasper-layout

布局引擎，核心中的核心。

**核心功能：**
- Band 展开（Detail band 按数据行重复）
- 分组逻辑（Group 表达式值变化时触发 header/footer）
- 分页算法（自动分页、PageHeader/Footer 重复、orphan/widow 控制）
- 元素绝对定位计算
- 文本自动换行和高度计算
- 跨页元素处理（表格跨页、子报表跨页）

**输出：** FilledDocument（包含多个 Page，每个 Page 包含绝对定位的元素列表）

**依赖：** xjasper-types, xjasper-expression, xjasper-variables, xjasper-datasource

#### xjasper-renderer-pdf

PDF 渲染器。

**核心功能：**
- 基于 printpdf 生成 PDF
- 字体嵌入和管理（rustybuzz + ttf-parser）
- PDF/A-1b 合规输出
- 水印和背景层
- 元数据和书签

**依赖：** xjasper-types, printpdf, lopdf, rustybuzz, ttf-parser

#### xjasper-renderer-image

图片渲染器。

**核心功能：**
- 基于 tiny-skia 渲染为 PNG/JPEG
- DPI 设置
- 抗锯齿和字体渲染

**依赖：** xjasper-types, tiny-skia, image

#### xjasper-renderer-html

HTML 渲染器（用于设计器预览）。

**核心功能：**
- 生成 HTML + CSS
- 保持像素级精确（绝对定位）
- 支持浏览器端预览

**依赖：** xjasper-types

#### xjasper-engine

门面模块，串联完整管线。

**核心 API：**
```rust
pub struct ReportEngine {
    // ...
}

impl ReportEngine {
    pub fn new() -> Self;

    pub fn compile_template(&self, json: &str) -> Result<CompiledTemplate>;

    pub fn fill_report(
        &self,
        template: &CompiledTemplate,
        data: Box<dyn DataSource>,
        params: HashMap<String, Value>,
    ) -> Result<FilledDocument>;

    pub fn export_pdf(&self, doc: &FilledDocument) -> Result<Vec<u8>>;
    pub fn export_png(&self, doc: &FilledDocument, dpi: u32) -> Result<Vec<Vec<u8>>>;
    pub fn export_html(&self, doc: &FilledDocument) -> Result<String>;
}
```

**依赖：** 所有核心 crate

#### xjasper-wasm

WASM 绑定层。

**核心功能：**
- wasm-bindgen 导出 JS 可调用的 API
- 类型转换（Rust ↔ JS）
- 错误处理和异常映射
- 内存管理优化

**导出 API：**
```typescript
export function compileTemplate(json: string): Uint8Array;
export function fillReport(
  compiled: Uint8Array,
  data: any[],
  params: Record<string, any>
): Uint8Array;
export function exportPdf(filled: Uint8Array): Uint8Array;
export function exportPng(filled: Uint8Array, dpi: number): Uint8Array[];
```

**依赖：** xjasper-engine, wasm-bindgen, serde-wasm-bindgen

#### xjasper-cli

命令行工具。

**功能：**
```bash
xjasper compile template.json -o template.xjc
xjasper fill template.xjc data.json -o output.pdf
xjasper render template.json data.json -o output.pdf --format pdf
xjasper validate template.json
```

**依赖：** xjasper-engine, clap

---

## 4. 模板格式设计

### 4.1 JSON Schema 示例

```json
{
  "name": "financial_statement",
  "version": "1.0",
  "page": {
    "width": 595,
    "height": 842,
    "orientation": "portrait",
    "margins": { "top": 40, "right": 40, "bottom": 40, "left": 40 }
  },
  "parameters": [
    { "name": "reportTitle", "type": "string", "default": "Financial Report" },
    { "name": "reportDate", "type": "date", "default": null }
  ],
  "fields": [
    { "name": "accountName", "type": "string" },
    { "name": "amount", "type": "decimal" },
    { "name": "currency", "type": "string" }
  ],
  "variables": [
    {
      "name": "totalAmount",
      "type": "decimal",
      "calculation": "Sum",
      "expression": "$F{amount}",
      "resetType": "Report",
      "initialValue": "0"
    }
  ],
  "groups": [
    {
      "name": "currencyGroup",
      "expression": "$F{currency}",
      "startNewPage": false,
      "resetPageNumber": false,
      "reprintHeaderOnEachPage": true
    }
  ],
  "styles": [
    {
      "name": "headerStyle",
      "fontFamily": "Arial",
      "fontSize": 12,
      "bold": true,
      "foreColor": "#FFFFFF",
      "backColor": "#4472C4",
      "hAlign": "center",
      "vAlign": "middle"
    }
  ],
  "bands": {
    "title": {
      "height": 60,
      "elements": [
        {
          "type": "textField",
          "x": 0,
          "y": 10,
          "width": 515,
          "height": 40,
          "expression": "$P{reportTitle}",
          "style": "headerStyle"
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
          "expression": "$F{accountName}"
        },
        {
          "type": "textField",
          "x": 300,
          "y": 0,
          "width": 215,
          "height": 20,
          "expression": "$F{amount}",
          "pattern": "#,##0.00"
        }
      ]
    },
    "summary": {
      "height": 30,
      "elements": [
        {
          "type": "textField",
          "x": 300,
          "y": 5,
          "width": 215,
          "height": 20,
          "expression": "$V{totalAmount}",
          "pattern": "#,##0.00",
          "style": "headerStyle"
        }
      ]
    }
  }
}
```

### 4.2 表达式语法

XJasper 表达式基于 rhai，支持类 JavaScript 语法：

```javascript
// 字段引用
$F{accountName}

// 变量引用
$V{totalAmount}

// 参数引用
$P{reportTitle}

// 字符串拼接
$F{firstName} + " " + $F{lastName}

// 条件表达式
if $F{amount} > 1000 { "High" } else { "Low" }

// 数学运算
$F{price} * $F{quantity}

// 函数调用
format_number($F{amount}, "#,##0.00")
format_date($P{reportDate}, "yyyy-MM-dd")

// 空值处理
$F{name} ?? "N/A"
```

**内置函数库：**

| 函数 | 用途 | 示例 |
|------|------|------|
| format_number | 数字格式化 | `format_number(1234.5, "#,##0.00")` → "1,234.50" |
| format_date | 日期格式化 | `format_date(date, "yyyy-MM-dd")` → "2026-03-01" |
| format_currency | 货币格式化 | `format_currency(1234.5, "USD")` → "$1,234.50" |
| upper / lower | 大小写转换 | `upper("hello")` → "HELLO" |
| substring | 字符串截取 | `substring("hello", 0, 3)` → "hel" |
| round / floor / ceil | 数值取整 | `round(1.567, 2)` → 1.57 |
| abs | 绝对值 | `abs(-10)` → 10 |
| max / min | 最大/最小值 | `max(10, 20)` → 20 |

---

## 5. 金融领域特化设计

### 5.1 数值精度保证

**问题：** JavaScript 的 Number 类型是 IEEE 754 双精度浮点数，存在精度问题：
```javascript
0.1 + 0.2 === 0.30000000000000004  // true
```

**解决方案：** 使用 rust_decimal crate，128bit 定点数：

```rust
use rust_decimal::Decimal;

let a = Decimal::from_str("0.1").unwrap();
let b = Decimal::from_str("0.2").unwrap();
let c = a + b;
assert_eq!(c.to_string(), "0.3");  // 精确
```

**在 WASM 中的处理：**
- Rust 侧使用 Decimal 计算
- 传递给 JS 时转为字符串（避免精度丢失）
- JS 侧使用 decimal.js 或 big.js 展示

### 5.2 多币种格式化

基于 icu4x 实现国际化数字格式化：

```rust
use icu::decimal::FixedDecimalFormatter;
use icu::locid::locale;

// 美元格式
let formatter = FixedDecimalFormatter::try_new(
    &locale!("en-US").into(),
    Default::default()
).unwrap();
// 输出：$1,234.56

// 人民币格式
let formatter = FixedDecimalFormatter::try_new(
    &locale!("zh-CN").into(),
    Default::default()
).unwrap();
// 输出：¥1,234.56

// 欧元格式（千分位用空格）
let formatter = FixedDecimalFormatter::try_new(
    &locale!("fr-FR").into(),
    Default::default()
).unwrap();
// 输出：1 234,56 €
```

**支持的格式化选项：**
- 千分位分隔符（逗号、空格、点）
- 小数点符号（点、逗号）
- 货币符号位置（前缀、后缀）
- 负数表示法（负号、括号）
- 小数位数控制

### 5.3 PDF/A 合规

金融报表需要长期存档，必须符合 PDF/A 标准。

**PDF/A-1b 要求：**
- 所有字体必须嵌入
- 禁止加密
- 禁止外部内容引用
- 必须包含 XMP 元数据
- 颜色空间必须设备无关（sRGB/CMYK）

**实现方案：**
```rust
use printpdf::*;

let doc = PdfDocument::new("Financial Report");
doc.set_conformance(PdfConformance::PdfA1b);
doc.embed_font(&font_data);  // 强制嵌入字体
doc.set_metadata(PdfMetadata {
    title: Some("Financial Statement"),
    author: Some("XJasper"),
    subject: Some("Q4 2025 Report"),
    creator: Some("XJasper Engine v1.0"),
    producer: Some("XJasper Engine v1.0"),
    creation_date: Some(Utc::now()),
});
```

### 5.4 水印和签章

**水印层：**
- 在 background band 中渲染半透明文本/图片
- 支持旋转角度（如 45° 斜向水印）
- 支持密级标识（机密、秘密、内部）

**数字签章占位：**
- 预留签章区域（矩形框）
- 输出 PDF 后由外部签章系统填充
- 支持多签章（多级审批）

```json
{
  "bands": {
    "background": {
      "height": 842,
      "elements": [
        {
          "type": "image",
          "x": 200,
          "y": 300,
          "width": 200,
          "height": 100,
          "source": "watermark.png",
          "opacity": 0.1,
          "rotation": 45
        }
      ]
    },
    "summary": {
      "height": 100,
      "elements": [
        {
          "type": "rectangle",
          "x": 400,
          "y": 50,
          "width": 100,
          "height": 40,
          "borderColor": "#000000",
          "borderWidth": 1,
          "annotation": "signature_placeholder"
        }
      ]
    }
  }
}
```

### 5.5 大数据量优化

金融报表可能包含数万行明细数据，需要流式处理。

**优化策略：**

1. **流式数据源** — DataSource trait 的 next() 方法逐行读取，不一次性加载全部数据
2. **增量布局** — 布局引擎逐 Band 计算，当前页满了立即输出，不等全部数据
3. **分页渲染** — PDF 渲染器逐页写入，不在内存中构建完整文档
4. **变量状态压缩** — 只保留必要的聚合状态，不保留原始数据

**内存占用估算：**
- 10,000 行数据，每行 10 个字段，每字段平均 50 字节
- 传统方式：10,000 × 10 × 50 = 5MB（全部加载）
- 流式处理：当前行 + 变量状态 < 10KB

---

## 6. 开发路线

### Phase 1: Rust 核心链路（4-6 周）

**目标：** CLI 工具能从 JSON 模板 + JSON 数据生成 PDF

**任务清单：**
1. 搭建 Cargo workspace
2. xjasper-types — 定义核心类型
3. xjasper-expression — 表达式引擎（基于 rhai）
4. xjasper-template — JSON 模板解析
5. xjasper-compiler — 模板编译
6. xjasper-datasource — JSON 数据源实现
7. xjasper-variables — 变量聚合引擎
8. xjasper-layout — 布局引擎（核心）
9. xjasper-renderer-pdf — PDF 渲染（基于 printpdf）
10. xjasper-engine — 门面模块
11. xjasper-cli — 命令行工具
12. 集成测试 — 端到端测试用例

**验收标准：**
```bash
xjasper render examples/invoice.json examples/data.json -o output.pdf
# 生成的 PDF 包含正确的数据、分页、汇总
```

### Phase 2: WASM 封装（2-3 周）

**目标：** 浏览器端能调用引擎生成 PDF

**任务清单：**
1. xjasper-wasm — WASM 绑定层
2. 类型转换优化（Rust ↔ JS）
3. 错误处理和异常映射
4. 内存管理优化（避免内存泄漏）
5. npm 包发布（@xjasper/core）
6. 浏览器端示例（Vanilla JS + React）

**验收标准：**
```javascript
import { compileTemplate, fillReport, exportPdf } from '@xjasper/core';

const compiled = compileTemplate(templateJson);
const filled = fillReport(compiled, data, params);
const pdfBytes = exportPdf(filled);

// 下载 PDF
const blob = new Blob([pdfBytes], { type: 'application/pdf' });
const url = URL.createObjectURL(blob);
window.open(url);
```

### Phase 3: 设计器 UI（6-8 周）

**目标：** 可视化编辑模板，实时预览

**任务清单：**
1. 画布编辑器（React + Konva.js）
   - 拖拽创建元素
   - 选择、移动、缩放
   - 对齐辅助线
   - 多选和组操作
2. 属性面板
   - 元素属性编辑
   - 样式编辑
   - 表达式编辑器（语法高亮、自动补全）
3. Band 管理面板
   - Band 列表
   - Band 高度调整
   - Band 显示/隐藏
4. 数据源配置面板
   - 字段定义
   - 参数定义
   - 变量定义
   - 分组定义
5. 实时预览
   - 加载样本数据
   - 调用 WASM 引擎渲染
   - PDF 预览（pdf.js）
6. 模板导入/导出
   - JSON 文件上传/下载
   - 模板库（本地存储）

**验收标准：**
- 用户能通过拖拽创建一个完整的发票模板
- 加载样本数据后能实时预览 PDF
- 导出的 JSON 模板能被 CLI 工具正确渲染

### Phase 4: Tauri 桌面应用（3-4 周）

**目标：** 桌面端完整体验，原生文件系统访问

**任务清单：**
1. Tauri 2 项目搭建
2. 整合设计器 UI（复用 Phase 3 代码）
3. 原生文件系统访问（打开/保存模板）
4. 原生数据源支持（CSV、Excel、SQLite）
5. 打包和分发（Windows、macOS、Linux）
6. 自动更新机制

**验收标准：**
- 桌面应用能打开本地 JSON 模板
- 能连接本地 SQLite 数据库作为数据源
- 能导出 PDF 到本地文件系统
- 安装包大小 < 20MB

### Phase 5: 金融增强功能（4-6 周）

**目标：** 交叉表、子报表、条件样式、PDF/A、水印签章

**任务清单：**
1. 交叉表组件（Crosstab）
2. 子报表支持（Subreport）
3. 条件样式系统
4. PDF/A-1b 合规输出
5. 水印和签章占位
6. 图表组件（基于 plotters）
7. 条形码/二维码（基于 barcoders）
8. CSV/Excel 导出（基于 csv/calamine）

**验收标准：**
- 能生成包含交叉表的复杂财务报表
- 能嵌套子报表（如订单明细）
- 输出的 PDF 通过 PDF/A 验证工具检查
- 支持常见条形码和二维码

---

## 7. 技术风险与应对

### 7.1 WASM 包体大小

**风险：** 字体嵌入会让 WASM 包变大（单个中文字体 5-10MB）

**应对：**
- 字体不打包进 WASM，通过 JS 传入
- 提供字体子集化工具（只嵌入用到的字符）
- 使用 wasm-opt 优化 WASM 体积

### 7.2 WASM 性能

**风险：** WASM 性能可能不如原生 Rust

**应对：**
- 布局计算在 WASM 中执行（CPU 密集，WASM 接近原生）
- PDF 生成在 WASM 中执行（避免大量数据传输）
- 使用 wasm-bindgen 的 --target web 优化

### 7.3 浏览器兼容性

**风险：** 旧浏览器可能不支持 WASM

**应对：**
- 最低要求：Chrome 57+、Firefox 52+、Safari 11+、Edge 16+
- 提供降级方案：服务端渲染 API

### 7.4 Rust 开发效率

**风险：** Rust 开发速度比 TypeScript 慢

**应对：**
- 核心引擎一旦稳定很少改动，前期投入值得
- 设计器 UI 用 React，快速迭代
- 充分利用 Rust 生态（crates.io）

---

## 8. 总结

XJasper 通过 Rust 核心引擎 + WASM + Tauri 2 的架构，在金融报表场景下实现了：

1. **金融级数值精度** — Decimal128 定点数
2. **像素级精确布局** — 绝对定位，满足监管要求
3. **高性能渲染** — Rust 零 GC，大数据量流式处理
4. **跨平台复用** — 一套代码，Web + 桌面
5. **现代开发体验** — JSON 模板、可视化设计器

相比 JasperReports，XJasper 更轻量、更快、更现代；相比 pdfme/ReportBro，XJasper 功能更完整、性能更强。这是一个面向未来的报表引擎架构。