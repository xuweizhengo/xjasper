# XJasper 竞品分析报告

## 1. 项目背景

XJasper 旨在构建一个现代化的、面向金融领域的像素级精确报表引擎，包含核心模板引擎、渲染引擎和可视化设计工具。本文档对主要竞品进行技术分析，为架构设计提供参考。

---

## 2. JasperReports（核心竞品）

### 2.1 基本信息

- 诞生年份：2001
- 语言：Java
- 协议：LGPL / 商业双授权
- 最新版本：JasperReports 7.x（2025）
- 设计器：Jaspersoft Studio（基于 Eclipse RCP）

### 2.2 核心架构 — 四阶段管线

```
JRXML (.jrxml)  →  JasperDesign  →  JasperReport (.jasper)  →  JasperPrint (.jrprint)
    [解析]            [编译]              [填充]                   [导出]
```

| 阶段 | 输入 | 输出 | 核心类 |
|------|------|------|--------|
| 解析 | JRXML (XML) | JasperDesign（可变内存对象） | JRXmlLoader |
| 编译 | JasperDesign | JasperReport（不可变，含字节码） | JasperCompileManager |
| 填充 | JasperReport + 数据源 | JasperPrint（绝对定位页面） | JasperFillManager |
| 导出 | JasperPrint | PDF / HTML / Excel / 图片等 | JasperExportManager |

设计精妙之处：编译产物（.jasper）可缓存复用，填充和导出完全解耦。同一模板编译一次，可用不同数据反复填充。

### 2.3 Band 模型

JasperReports 采用水平条带（Band）布局模型，每个 Band 是一个水平区域，在特定时机出现：

| Band | 出现时机 | 典型用途 |
|------|---------|---------|
| background | 每页底层 | 水印、背景图 |
| title | 报表开头一次 | 报表标题、Logo |
| pageHeader | 每页顶部 | 页眉、报表名称 |
| columnHeader | 每列顶部 | 表头列名 |
| groupHeader | 分组值变化时 | 分组标题 |
| detail | 每条数据一次 | 数据行（核心重复区域） |
| groupFooter | 分组结束时 | 分组小计 |
| columnFooter | 每列底部 | 列汇总 |
| pageFooter | 每页底部 | 页码、时间戳 |
| lastPageFooter | 最后一页底部 | 替代 pageFooter |
| summary | 所有数据之后 | 总计、图表 |
| noData | 数据源为空时 | "无数据"提示 |

每个 Band 有 height 属性，可包含任意数量的报表元素，支持 `printWhenExpression` 条件显示。

### 2.4 表达式系统

三种前缀区分数据来源：

```
$F{fieldName}     — 数据源字段
$V{variableName}  — 计算变量（Sum、Count、Avg 等）
$P{parameterName} — 外部参数
$R{key}           — 国际化资源
```

表达式本质是 Java/Groovy 代码片段，编译时生成字节码：

```xml
<!-- 简单字段引用 -->
<textFieldExpression><![CDATA[$F{name}]]></textFieldExpression>

<!-- 字符串拼接 -->
<textFieldExpression><![CDATA[$F{firstName} + " " + $F{lastName}]]></textFieldExpression>

<!-- 条件表达式 -->
<textFieldExpression><![CDATA[$F{amount}.doubleValue() > 1000 ? "High" : "Low"]]></textFieldExpression>
```

### 2.5 数据源抽象

核心接口极简：

```java
public interface JRDataSource {
    boolean next() throws JRException;
    Object getFieldValue(JRField jrField) throws JRException;
}
```

内置适配器：

| 实现类 | 数据源 |
|--------|--------|
| JRResultSetDataSource | JDBC ResultSet |
| JRBeanCollectionDataSource | JavaBean 集合 |
| JRMapCollectionDataSource | Map 集合 |
| JRXmlDataSource | XML (XPath) |
| JRCsvDataSource | CSV 文件 |
| JRXlsDataSource | Excel 文件 |
| JREmptyDataSource | 空数据源（测试用） |

### 2.6 变量与聚合

变量支持的聚合类型：

| 类型 | 行为 |
|------|------|
| Sum | 累加求和 |
| Count | 非空值计数 |
| DistinctCount | 去重计数 |
| Average | 平均值 |
| Lowest / Highest | 最小/最大值 |
| StandardDeviation | 标准差 |
| Variance | 方差 |
| First | 首条记录值 |

每个变量有 resetType（何时重置：Report/Page/Column/Group）和 incrementType（何时累加）。

内置变量：PAGE_NUMBER、COLUMN_NUMBER、REPORT_COUNT、PAGE_COUNT、{GroupName}_COUNT。

### 2.7 元素类型

| 元素 | 用途 |
|------|------|
| staticText | 固定文本 |
| textField | 动态文本（表达式） |
| image | 静态/动态图片 |
| line / rectangle / ellipse | 几何形状 |
| subreport | 嵌套子报表 |
| frame | 元素容器 |
| chart | 图表（JFreeChart） |
| crosstab | 交叉表/透视表 |
| table | 表格组件 |
| barcode | 条形码/二维码 |
| break | 分页/分列符 |

### 2.8 导出格式

PDF、HTML、XLSX、XLS、CSV、XML、DOCX、PPTX、ODT、ODS、RTF、纯文本、JSON、Graphics2D、打印服务。

### 2.9 样式系统

- 样式继承：子样式通过 `style` 属性引用父样式
- 条件样式：根据表达式动态切换（如负数红色）
- 外部样式模板：.jrtx 文件跨报表共享
- 默认样式：`isDefault="true"` 应用于未指定样式的元素

### 2.10 JasperReports 的优势与不足

**优势：**
- 20+ 年积累，功能最完整的开源报表引擎
- Band 模型 + Group + Variable 体系成熟
- 子报表、交叉表等高级功能完备
- 导出格式最全

**不足：**
- 重 Java 依赖，部署需要 JVM，容器化笨重
- JRXML 冗长，版本控制 diff 困难
- 表达式编译为 Java 字节码，过于重量级
- 设计器基于 Eclipse，体验停留在桌面时代
- 无法浏览器端运行，预览必须走服务端
- 扩展自定义元素需实现大量 Java 接口

---

## 3. 现代竞品分析

### 3.1 pdfme（TypeScript，MIT 开源）

**定位：** 基于 JSON 模板的 PDF 生成库，含 WYSIWYG 设计器。

**架构：**
- `@pdfme/common` — 共享类型和工具
- `@pdfme/generator` — PDF 生成引擎（基于 pdf-lib）
- `@pdfme/ui` — React 设计器 + 查看器

**技术特点：**
- JSON 模板格式，Schema 驱动
- 浏览器和 Node.js 双端运行
- React 设计器使用 react-moveable 实现拖拽
- 支持 basePdf 叠加（在已有 PDF 上填充数据）

**优势：** TypeScript 全栈、模块化清晰、设计器体验现代
**不足：** 没有 Band 模型、没有分组聚合、没有分页逻辑。本质是"PDF 表单填充工具"，不是报表引擎。无法处理多页数据报表。

**对 XJasper 的参考价值：** JSON 模板格式设计、模块化包结构、双端运行架构。

### 3.2 ReportBro（Python + JS，开源）

**定位：** 像素级精确的 PDF 报表生成器，含浏览器端设计器。

**架构：**
- 前端：Vue.js 可视化设计器
- 后端：Python 渲染库（reportbro-lib）
- 交换格式：JSON 模板定义

**技术特点：**
- 支持表格、分组、条件渲染、分页
- 像素级精确定位（键盘逐像素移动）
- 表达式使用 Python 语法
- 支持条形码（Code128、QR 等）

**元素类型：** 文本、图片、表格（含分组）、Section、Frame、条形码、线条、分页符

**优势：** 像素级精确、浏览器端设计器体验好
**不足：** Python 后端（不适合 JS/TS 全栈）、前后端语言割裂、没有变量聚合体系、分组能力弱于 JasperReports

**对 XJasper 的参考价值：** 浏览器端设计器交互模式、像素级定位实现。

### 3.3 Typst（Rust，开源）

**定位：** 现代排版系统，LaTeX 的替代品，可用于自动化 PDF 生成。

**架构：**
- 单一 Rust 二进制（~40MB）
- 内置脚本语言做数据转换
- 原生 JSON/CSV/XML 数据加载
- 通过 `sys.inputs` 传入外部数据

**技术特点：**
- 毫秒级编译速度
- 原生 PDF/A、PDF/UA 合规（无障碍）
- Docker 友好，适合大规模自动化
- 300+ 贡献者，活跃开发中
- 字体处理：rustybuzz + ttf-parser

**优势：** 极高性能、PDF 标准合规、Rust 生态成熟
**不足：** 不是报表引擎而是排版系统、没有可视化设计器、模板是代码不是配置

**对 XJasper 的参考价值：** Rust PDF 渲染技术栈验证、字体处理方案（rustybuzz + ttf-parser）、PDF/A 合规实现、高性能架构参考。

### 3.4 CSS Paged Media（WeasyPrint / Prince XML）

**定位：** 用 HTML + CSS 生成分页 PDF。

**技术特点：**
- `@page` 规则控制页面尺寸、页眉页脚
- CSS `break-before` / `break-after` 控制分页
- WeasyPrint 开源免费（Python），Prince XML 商业

**优势：** Web 开发者零学习成本、CSS 生态丰富
**不足：** CSS 分页控制能力有限、复杂分组和跨页表格难以实现、像素级精确定位困难

**对 XJasper 的参考价值：** 有限。CSS 布局模型不适合金融报表的强格式需求。

### 3.5 Stimulsoft（商业）

**定位：** 全功能商业报表平台。

**技术特点：**
- 多平台：.NET / JS / Java / PHP
- Web 设计器 + 200+ 导出格式
- 交叉表、仪表盘、图表全支持
- 2026 年仍在活跃更新

**优势：** 功能最全的商业方案
**不足：** 商业授权昂贵、代码不可控、定制受限

**对 XJasper 的参考价值：** 功能完整度的标杆参考。

---

## 4. 竞品对比矩阵

| 特性 | JasperReports | pdfme | ReportBro | Typst | Stimulsoft | XJasper（目标） |
|------|:---:|:---:|:---:|:---:|:---:|:---:|
| Band 模型 | ✅ 完整 | ❌ | ⚠️ 基础 | ❌ | ✅ | ✅ 完整 |
| 分组聚合 | ✅ | ❌ | ⚠️ 基础 | ❌ | ✅ | ✅ |
| 交叉表 | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ Phase 5 |
| 子报表 | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ Phase 5 |
| 可视化设计器 | ⚠️ 桌面 | ✅ Web | ✅ Web | ❌ | ✅ Web | ✅ Web + 桌面 |
| 浏览器端渲染 | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ WASM |
| 像素级精确 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 金融数值精度 | ⚠️ BigDecimal | ❌ float | ❌ | ❌ | ⚠️ | ✅ Decimal128 |
| PDF/A 合规 | ⚠️ 有限 | ❌ | ❌ | ✅ | ✅ | ✅ |
| 模板格式 | XML | JSON | JSON | 代码 | XML | JSON |
| 性能 | 中等 | 低 | 中等 | 极高 | 中等 | 高（Rust） |
| 部署体积 | 重（JVM） | 轻 | 中等 | 轻（40MB） | 中等 | 轻 |

---

## 5. 结论与设计方向

### 5.1 从各竞品吸收的设计要素

| 来源 | 吸收要素 |
|------|---------|
| JasperReports | 四阶段管线、完整 Band 模型、Group + Variable 体系、子报表、交叉表 |
| pdfme | JSON 模板格式、模块化包结构、浏览器/Node.js 双端架构 |
| ReportBro | 浏览器端设计器交互、像素级定位 |
| Typst | Rust 核心引擎、字体处理方案、PDF/A 合规、高性能架构 |

### 5.2 XJasper 的差异化定位

**"现代化的、Rust 驱动的、面向金融场景的像素级精确报表引擎"**

核心差异点：
1. Rust 核心引擎 → 性能和数值精度碾压 Java/JS 方案
2. WASM 编译 → 浏览器端原生运行，无需服务端渲染
3. Tauri 2 → 一套代码同时支持 Web 和桌面
4. JSON 模板 → 比 XML 更轻量，Git 友好
5. 金融特化 → Decimal128 精度、多币种格式化、PDF/A 合规、水印签章
