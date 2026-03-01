# XJasper

> 现代化的、Rust 驱动的、面向金融场景的像素级精确报表引擎

## 项目简介

XJasper 是一个高性能报表引擎，旨在替代 JasperReports，提供现代化的开发体验和金融级的数值精度。

### 核心特性

- **Rust 核心引擎** — 零 GC、高性能、内存安全
- **金融级数值精度** — Decimal128 定点数，避免浮点误差
- **像素级精确布局** — 绝对定位，满足监管报表格式要求
- **跨平台复用** — 一套代码，WASM 浏览器端 + Tauri 桌面端
- **完整的 Band 模型** — 借鉴 JasperReports，支持分组、聚合、分页
- **可视化设计器** — Web + 桌面双端，拖拽式编辑
- **JSON 模板** — 比 XML 更轻量，Git 友好
- **PDF/A 合规** — 支持长期存档标准

### 技术栈

| 层面 | 技术 |
|------|------|
| 核心引擎 | Rust |
| 数值计算 | rust_decimal |
| PDF 生成 | printpdf + lopdf |
| 字体处理 | rustybuzz + ttf-parser |
| 图片渲染 | tiny-skia |
| 表达式引擎 | rhai |
| WASM 绑定 | wasm-bindgen |
| 桌面框架 | Tauri 2 |
| 设计器 UI | React + Konva.js |

---

## 项目结构

```
xjasper/
├── docs/                          # 文档
│   ├── 01-competitive-analysis.md # 竞品分析
│   ├── 02-technical-design.md     # 技术设计
│   ├── 03-financial-requirements.md # 金融需求分析
│   └── 04-development-guide.md    # 开发指南
├── crates/                        # Rust 核心（Cargo workspace）
│   ├── xjasper-types/             # 类型定义
│   ├── xjasper-expression/        # 表达式引擎
│   ├── xjasper-template/          # 模板解析
│   ├── xjasper-compiler/          # 模板编译
│   ├── xjasper-datasource/        # 数据源抽象
│   ├── xjasper-variables/         # 变量聚合引擎
│   ├── xjasper-layout/            # 布局引擎
│   ├── xjasper-renderer-pdf/      # PDF 渲染
│   ├── xjasper-renderer-image/    # 图片渲染
│   ├── xjasper-renderer-html/     # HTML 渲染
│   ├── xjasper-engine/            # 门面模块
│   ├── xjasper-wasm/              # WASM 绑定
│   └── xjasper-cli/               # CLI 工具
├── apps/
│   ├── desktop/                   # Tauri 2 桌面应用
│   └── web/                       # 纯 Web 版
├── packages/
│   └── designer-ui/               # 设计器 UI 组件
├── examples/                      # 示例模板和数据
├── Cargo.toml                     # Workspace root
└── package.json                   # 前端 monorepo
```

---

## 快速开始

### 环境要求

- Rust 1.75+
- Node.js 18+
- pnpm 8+

### 安装依赖

```bash
# Rust 依赖
cargo build

# 前端依赖
pnpm install
```

### 运行示例

```bash
# CLI 工具渲染示例
cargo run --bin xjasper-cli -- render examples/invoice.json examples/data.json -o output.pdf

# 启动 Web 设计器
cd apps/web
pnpm dev

# 启动桌面应用
cd apps/desktop
pnpm tauri dev
```

---

## 开发路线

### Phase 1: Rust 核心链路（当前）

- [x] 项目文档编写
- [ ] Cargo workspace 搭建
- [ ] 核心类型定义（xjasper-types）
- [ ] 表达式引擎（xjasper-expression）
- [ ] 模板解析（xjasper-template）
- [ ] 布局引擎（xjasper-layout）
- [ ] PDF 渲染（xjasper-renderer-pdf）
- [ ] CLI 工具（xjasper-cli）

**目标：** CLI 工具能从 JSON 模板 + JSON 数据生成 PDF

### Phase 2: WASM 封装

- [ ] WASM 绑定层（xjasper-wasm）
- [ ] npm 包发布（@xjasper/core）
- [ ] 浏览器端示例

**目标：** 浏览器端能调用引擎生成 PDF

### Phase 3: 设计器 UI

- [ ] 画布编辑器（React + Konva.js）
- [ ] 属性面板
- [ ] 实时预览
- [ ] 模板导入/导出

**目标：** 可视化编辑模板，实时预览

### Phase 4: Tauri 桌面应用

- [ ] Tauri 2 项目搭建
- [ ] 原生文件系统访问
- [ ] 打包和分发

**目标：** 桌面端完整体验

### Phase 5: 金融增强功能

- [ ] 交叉表（Crosstab）
- [ ] 子报表（Subreport）
- [ ] 条件样式
- [ ] PDF/A 合规
- [ ] 水印和签章
- [ ] 图表组件
- [ ] 条形码/二维码

**目标：** 完整的金融报表功能

---

## 文档

- [竞品分析](docs/01-competitive-analysis.md) — JasperReports 和现代竞品的技术分析
- [技术设计](docs/02-technical-design.md) — 架构设计、模块拆分、技术选型
- [金融需求分析](docs/03-financial-requirements.md) — 金融领域的特殊需求
- [开发指南](docs/04-development-guide.md) — 开发环境搭建、编码规范、测试指南

---

## 贡献

欢迎贡献代码、报告问题、提出建议。

### 开发流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码规范

- Rust 代码遵循 `rustfmt` 和 `clippy` 规范
- TypeScript 代码遵循 ESLint + Prettier 规范
- 提交信息遵循 Conventional Commits

---

## 许可证

MIT License

---

## 致谢

- [JasperReports](https://github.com/TIBCOSoftware/jasperreports) — 报表引擎设计参考
- [Typst](https://github.com/typst/typst) — Rust PDF 渲染技术参考
- [pdfme](https://github.com/pdfme/pdfme) — JSON 模板格式参考
- [ReportBro](https://www.reportbro.com/) — 设计器交互参考