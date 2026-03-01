# XJasper 金融领域需求分析

## 1. 金融报表的特殊性

金融报表与普通报表的核心差异在于**强格式要求**和**数值精度要求**。这些要求来自监管合规、审计追溯、长期存档等场景。

---

## 2. 核心需求分类

### 2.1 格式精度需求

#### 2.1.1 像素级精确定位

**需求描述：**
监管报表（如银行业 Basel III 报告、保险业 Solvency II 报告）有固定的格式模板，每个字段的位置、字体、大小都有明确规定，不能有任何偏差。

**技术要求：**
- 绝对定位，不能使用流式布局
- 坐标精度到像素（1px = 1/72 inch）
- 跨页元素位置必须精确对齐
- 表格线条粗细必须可控（0.5pt、1pt、2pt）

**实现方案：**
- 所有元素使用 (x, y, width, height) 绝对坐标
- 布局引擎不做自动调整，严格按模板定义渲染
- 提供网格和辅助线帮助设计器精确定位

#### 2.1.2 字体和样式一致性

**需求描述：**
同一类型的报表必须使用统一的字体、字号、颜色，确保视觉一致性。

**技术要求：**
- 样式模板系统（类似 CSS 的 class）
- 样式继承和覆盖
- 条件样式（如负数红色、超阈值高亮）
- 字体嵌入（避免不同环境字体缺失）

**实现方案：**
- 样式定义在模板顶层，元素通过 style 属性引用
- 支持条件样式：`if $F{amount} < 0 then redStyle else normalStyle`
- PDF 输出时强制嵌入字体（PDF/A 要求）

#### 2.1.3 分页控制

**需求描述：**
- 特定内容必须在同一页（如签章区域）
- 表格跨页时表头必须重复
- 避免孤行/寡行（orphan/widow）
- 最后一页可能有特殊页脚（如审批签字栏）

**技术要求：**
- Band 级别的分页控制（startNewPage、keepTogether）
- 表格跨页时自动重复表头
- orphan/widow 控制（最少行数）
- lastPageFooter 特殊处理

**实现方案：**
- Band 属性：`keepTogether: true`（整个 Band 不跨页）
- Group 属性：`startNewPage: true`（分组开始时强制分页）
- 表格元素：`repeatHeader: true`（跨页重复表头）
- 布局引擎检测 orphan/widow，自动调整分页点

---

### 2.2 数值精度需求

#### 2.2.1 金额计算精度

**需求描述：**
金融计算不能有浮点误差，0.1 + 0.2 必须等于 0.3。

**问题示例：**
```javascript
// JavaScript 浮点数问题
0.1 + 0.2 === 0.3  // false
0.1 + 0.2 === 0.30000000000000004  // true

// 实际案例：汇总金额对不上
let sum = 0;
for (let i = 0; i < 1000; i++) {
  sum += 0.01;  // 累加 1000 次 0.01
}
console.log(sum);  // 9.999999999999831（不是 10）
```

**技术要求：**
- 使用定点数或高精度十进制类型
- 加减乘除运算精确
- 四舍五入可控（银行家舍入法）

**实现方案：**
- Rust 侧使用 `rust_decimal::Decimal`（128bit 定点数）
- 支持 28-29 位有效数字
- 四舍五入模式可配置：
  - RoundHalfUp（标准四舍五入）
  - RoundHalfEven（银行家舍入，0.5 舍入到最近的偶数）
  - RoundDown（向下取整）
  - RoundUp（向上取整）

```rust
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

let a = Decimal::from_str("0.1").unwrap();
let b = Decimal::from_str("0.2").unwrap();
let c = a + b;
assert_eq!(c, Decimal::from_str("0.3").unwrap());  // 精确

// 银行家舍入
let x = Decimal::from_str("2.5").unwrap();
let y = x.round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven);
assert_eq!(y, Decimal::from_str("2").unwrap());  // 2.5 → 2（偶数）

let x = Decimal::from_str("3.5").unwrap();
let y = x.round_dp_with_strategy(0, RoundingStrategy::MidpointNearestEven);
assert_eq!(y, Decimal::from_str("4").unwrap());  // 3.5 → 4（偶数）
```

#### 2.2.2 多币种格式化

**需求描述：**
同一报表可能展示多种货币，每种货币的格式不同。

**格式差异：**

| 货币 | 符号 | 千分位 | 小数点 | 示例 |
|------|------|--------|--------|------|
| 美元 USD | $ | 逗号 | 点 | $1,234.56 |
| 人民币 CNY | ¥ | 逗号 | 点 | ¥1,234.56 |
| 欧元 EUR | € | 空格 | 逗号 | 1 234,56 € |
| 日元 JPY | ¥ | 逗号 | 无小数 | ¥1,235 |
| 英镑 GBP | £ | 逗号 | 点 | £1,234.56 |

**技术要求：**
- 支持 ISO 4217 货币代码
- 自动应用对应的格式规则
- 支持自定义格式模板

**实现方案：**
- 使用 icu4x 的 FixedDecimalFormatter
- 内置常见货币格式
- 表达式函数：`format_currency($F{amount}, $F{currency})`

```rust
use icu::decimal::FixedDecimalFormatter;
use icu::locid::locale;

// 美元
let usd_formatter = FixedDecimalFormatter::try_new(
    &locale!("en-US").into(),
    Default::default()
).unwrap();

// 欧元（法国格式）
let eur_formatter = FixedDecimalFormatter::try_new(
    &locale!("fr-FR").into(),
    Default::default()
).unwrap();
```

#### 2.2.3 负数表示法

**需求描述：**
金融报表中负数有多种表示方式，需要可配置。

**常见表示法：**
- 负号前缀：`-1,234.56`
- 括号：`(1,234.56)`
- 红色：`1,234.56`（红色字体）
- 负号 + 红色：`-1,234.56`（红色字体）

**实现方案：**
- 样式属性：`negativeFormat: "parentheses" | "minus" | "color"`
- 条件样式：`if $F{amount} < 0 then negativeStyle`

```json
{
  "styles": [
    {
      "name": "negativeAmount",
      "foreColor": "#FF0000",
      "pattern": "(#,##0.00)"
    }
  ],
  "elements": [
    {
      "type": "textField",
      "expression": "$F{amount}",
      "conditionalStyles": [
        {
          "condition": "$F{amount} < 0",
          "style": "negativeAmount"
        }
      ]
    }
  ]
}
```

---

### 2.3 数据聚合需求

#### 2.3.1 多级分组汇总

**需求描述：**
金融报表常需要按多个维度分组汇总，如：机构 → 部门 → 产品线。

**示例：**
```
总行
  ├─ 零售银行部
  │   ├─ 个人贷款
  │   │   ├─ 明细1: 10,000
  │   │   ├─ 明细2: 20,000
  │   │   └─ 小计: 30,000
  │   ├─ 信用卡
  │   │   ├─ 明细3: 15,000
  │   │   └─ 小计: 15,000
  │   └─ 部门小计: 45,000
  ├─ 公司银行部
  │   └─ 部门小计: 80,000
  └─ 总计: 125,000
```

**技术要求：**
- 支持嵌套分组（Group 嵌套）
- 每个分组级别有独立的 header/footer
- 变量可以按分组重置（resetType: Group）

**实现方案：**
```json
{
  "groups": [
    {
      "name": "departmentGroup",
      "expression": "$F{department}",
      "header": { "height": 25, "elements": [...] },
      "footer": { "height": 20, "elements": [
        { "type": "textField", "expression": "$V{departmentTotal}" }
      ]}
    },
    {
      "name": "productGroup",
      "expression": "$F{product}",
      "parentGroup": "departmentGroup",
      "header": { "height": 20, "elements": [...] },
      "footer": { "height": 15, "elements": [
        { "type": "textField", "expression": "$V{productTotal}" }
      ]}
    }
  ],
  "variables": [
    {
      "name": "productTotal",
      "calculation": "Sum",
      "expression": "$F{amount}",
      "resetType": "Group",
      "resetGroup": "productGroup"
    },
    {
      "name": "departmentTotal",
      "calculation": "Sum",
      "expression": "$F{amount}",
      "resetType": "Group",
      "resetGroup": "departmentGroup"
    },
    {
      "name": "grandTotal",
      "calculation": "Sum",
      "expression": "$F{amount}",
      "resetType": "Report"
    }
  ]
}
```

#### 2.3.2 交叉表（Crosstab）

**需求描述：**
行列动态交叉的透视表，如月份 × 产品线的销售额矩阵。

**示例：**
```
产品线    | 1月    | 2月    | 3月    | 合计
---------|--------|--------|--------|--------
个人贷款  | 10,000 | 12,000 | 11,000 | 33,000
信用卡    | 8,000  | 9,000  | 8,500  | 25,500
合计      | 18,000 | 21,000 | 19,500 | 58,500
```

**技术要求：**
- 行分组、列分组、度量值
- 行小计、列小计、总计
- 单元格条件格式

**实现方案：**
```json
{
  "type": "crosstab",
  "rowGroups": [
    { "field": "product", "header": "产品线" }
  ],
  "columnGroups": [
    { "field": "month", "header": "月份" }
  ],
  "measures": [
    { "field": "amount", "aggregation": "Sum", "pattern": "#,##0" }
  ],
  "showRowTotals": true,
  "showColumnTotals": true
}
```

---

### 2.4 合规性需求

#### 2.4.1 PDF/A 长期存档

**需求描述：**
金融报表需要保存 7-30 年，必须符合 PDF/A 标准，确保未来可读。

**PDF/A-1b 要求：**
- 所有字体必须嵌入（不能依赖系统字体）
- 禁止加密（确保可访问）
- 禁止外部内容引用（如外部图片链接）
- 必须包含 XMP 元数据
- 颜色空间必须设备无关（sRGB/CMYK，不能用 RGB）

**实现方案：**
```rust
use printpdf::*;

let doc = PdfDocument::new("Financial Report");
doc.set_conformance(PdfConformance::PdfA1b);

// 强制嵌入字体
doc.embed_font(&font_data);

// 设置元数据
doc.set_metadata(PdfMetadata {
    title: Some("Q4 2025 Financial Statement"),
    author: Some("Finance Department"),
    subject: Some("Quarterly Report"),
    keywords: Some(vec!["finance", "Q4", "2025"]),
    creator: Some("XJasper Engine v1.0"),
    producer: Some("XJasper Engine v1.0"),
    creation_date: Some(Utc::now()),
    modification_date: Some(Utc::now()),
});

// 使用设备无关颜色空间
let color = Color::Srgb(Srgb::new(0.0, 0.0, 0.0, None));
```

#### 2.4.2 审计追溯

**需求描述：**
报表生成过程需要可追溯，包括模板版本、数据来源、生成时间、生成人。

**技术要求：**
- 模板版本化（Git 管理）
- 数据快照（生成时保存原始数据）
- 生成日志（时间戳、用户、参数）
- PDF 元数据记录（模板版本、数据哈希）

**实现方案：**
```json
{
  "template": {
    "name": "financial_statement",
    "version": "2.1.3",
    "lastModified": "2026-02-15T10:30:00Z",
    "author": "finance-team"
  }
}
```

PDF 元数据：
```rust
doc.set_custom_metadata("TemplateVersion", "2.1.3");
doc.set_custom_metadata("DataHash", "sha256:abc123...");
doc.set_custom_metadata("GeneratedBy", "user@company.com");
doc.set_custom_metadata("GeneratedAt", "2026-03-01T14:25:00Z");
```

#### 2.4.3 水印和密级标识

**需求描述：**
内部报表需要水印（如"内部资料"、"机密"），防止外泄。

**技术要求：**
- 半透明文本水印
- 旋转角度（如 45° 斜向）
- 重复平铺或单个居中
- 密级标识（机密、秘密、内部、公开）

**实现方案：**
```json
{
  "bands": {
    "background": {
      "height": 842,
      "elements": [
        {
          "type": "text",
          "x": 200,
          "y": 400,
          "width": 200,
          "height": 50,
          "text": "内部资料",
          "fontSize": 48,
          "foreColor": "#CCCCCC",
          "opacity": 0.2,
          "rotation": 45
        }
      ]
    }
  }
}
```

#### 2.4.4 数字签章占位

**需求描述：**
报表需要多级审批签章，PDF 生成时预留签章区域。

**技术要求：**
- 矩形占位框
- 标注签章类型（制表人、复核人、审批人）
- 输出 PDF 后由外部签章系统填充

**实现方案：**
```json
{
  "bands": {
    "summary": {
      "height": 100,
      "elements": [
        {
          "type": "rectangle",
          "x": 100,
          "y": 50,
          "width": 80,
          "height": 40,
          "borderColor": "#000000",
          "borderWidth": 1,
          "annotation": {
            "type": "signature_placeholder",
            "role": "preparer",
            "label": "制表人"
          }
        },
        {
          "type": "rectangle",
          "x": 200,
          "y": 50,
          "width": 80,
          "height": 40,
          "borderColor": "#000000",
          "borderWidth": 1,
          "annotation": {
            "type": "signature_placeholder",
            "role": "reviewer",
            "label": "复核人"
          }
        },
        {
          "type": "rectangle",
          "x": 300,
          "y": 50,
          "width": 80,
          "height": 40,
          "borderColor": "#000000",
          "borderWidth": 1,
          "annotation": {
            "type": "signature_placeholder",
            "role": "approver",
            "label": "审批人"
          }
        }
      ]
    }
  }
}
```

---

### 2.5 性能需求

#### 2.5.1 大数据量处理

**需求描述：**
金融报表可能包含数万行明细数据（如全年交易流水）。

**性能指标：**
- 10,000 行数据，生成时间 < 5 秒
- 100,000 行数据，生成时间 < 30 秒
- 内存占用 < 500MB（不随数据量线性增长）

**实现方案：**
- 流式数据源（DataSource trait 逐行读取）
- 增量布局（逐 Band 计算，当前页满了立即输出）
- 分页渲染（PDF 逐页写入，不在内存中构建完整文档）
- 变量状态压缩（只保留聚合状态，不保留原始数据）

#### 2.5.2 并发渲染

**需求描述：**
批量生成报表时（如月末批量生成客户对账单），需要并发处理。

**技术要求：**
- 多线程并行渲染
- 无共享状态（每个报表独立）
- 资源池管理（字体、图片缓存）

**实现方案：**
```rust
use rayon::prelude::*;

let reports: Vec<_> = customers.par_iter()
    .map(|customer| {
        let engine = ReportEngine::new();
        let data = load_customer_data(customer.id);
        engine.render(template, data)
    })
    .collect();
```

---

## 3. 典型应用场景

### 3.1 银行对账单

**特点：**
- 固定格式（监管要求）
- 大量明细数据（数千笔交易）
- 多币种（外币账户）
- 分页表头重复
- 末页签章区域

**模板要素：**
- pageHeader：银行 Logo、客户信息、账号
- detail：交易明细（日期、摘要、借方、贷方、余额）
- summary：期初余额、期末余额、利息
- lastPageFooter：签章占位

### 3.2 财务报表（资产负债表、利润表）

**特点：**
- 严格的格式规范（会计准则）
- 多级分组（科目层级）
- 同比/环比列
- 负数括号表示
- PDF/A 存档

**模板要素：**
- title：报表名称、报告期
- groupHeader：一级科目
- detail：明细科目
- groupFooter：科目小计
- summary：总计

### 3.3 监管报表（Basel III、Solvency II）

**特点：**
- 像素级精确（监管模板）
- 复杂计算（风险加权资产）
- 交叉表（风险矩阵）
- 多语言（中英文双语）
- 水印（机密标识）

**模板要素：**
- 固定表格布局
- 交叉表组件
- 条件格式（超阈值高亮）
- 水印层

---

## 4. 需求优先级

| 需求 | 优先级 | Phase |
|------|:------:|:-----:|
| 像素级精确定位 | P0 | Phase 1 |
| 金额计算精度（Decimal） | P0 | Phase 1 |
| Band 模型 + 分组聚合 | P0 | Phase 1 |
| PDF 导出 | P0 | Phase 1 |
| 多币种格式化 | P1 | Phase 1 |
| 条件样式 | P1 | Phase 5 |
| PDF/A 合规 | P1 | Phase 5 |
| 水印和签章 | P1 | Phase 5 |
| 交叉表 | P2 | Phase 5 |
| 子报表 | P2 | Phase 5 |
| 图表组件 | P3 | Phase 5+ |
| 条形码/二维码 | P3 | Phase 5+ |

---

## 5. 总结

金融领域的报表需求远超普通报表，核心在于：

1. **格式精度** — 像素级定位、样式一致性、分页控制
2. **数值精度** — Decimal 定点数、多币种格式化、负数表示法
3. **数据聚合** — 多级分组、交叉表、变量聚合
4. **合规性** — PDF/A 存档、审计追溯、水印签章
5. **性能** — 大数据量流式处理、并发渲染

XJasper 通过 Rust 核心引擎 + 完整的 Band 模型 + 金融特化功能，能够满足这些严苛的需求。