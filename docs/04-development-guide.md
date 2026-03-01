# XJasper 开发指南

## 1. 开发环境搭建

### 1.1 系统要求

- **操作系统：** Linux / macOS / Windows
- **Rust：** 1.75+ (stable)
- **Node.js：** 18+
- **pnpm：** 8+
- **Git：** 2.30+

### 1.2 安装 Rust

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# 下载并运行 https://rustup.rs/

# 验证安装
rustc --version
cargo --version
```

### 1.3 安装 Node.js 和 pnpm

```bash
# 使用 nvm 安装 Node.js
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# 安装 pnpm
npm install -g pnpm

# 验证安装
node --version
pnpm --version
```

### 1.4 克隆仓库

```bash
git clone https://github.com/your-org/xjasper.git
cd xjasper
```

### 1.5 安装依赖

```bash
# Rust 依赖（自动下载）
cargo build

# 前端依赖
pnpm install
```

---

## 2. 项目结构详解

### 2.1 Cargo Workspace

`Cargo.toml` 定义了 workspace 结构：

```toml
[workspace]
members = [
    "crates/xjasper-types",
    "crates/xjasper-expression",
    "crates/xjasper-template",
    "crates/xjasper-compiler",
    "crates/xjasper-datasource",
    "crates/xjasper-variables",
    "crates/xjasper-layout",
    "crates/xjasper-renderer-pdf",
    "crates/xjasper-renderer-image",
    "crates/xjasper-renderer-html",
    "crates/xjasper-engine",
    "crates/xjasper-wasm",
    "crates/xjasper-cli",
]

[workspace.dependencies]
# 共享依赖版本
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rust_decimal = "1.33"
chrono = "0.4"
thiserror = "1.0"
anyhow = "1.0"
```

### 2.2 Crate 依赖关系

```
xjasper-types (零依赖)
  ├── xjasper-expression
  ├── xjasper-template
  ├── xjasper-compiler ← expression, template
  ├── xjasper-datasource
  ├── xjasper-variables ← expression
  └── xjasper-layout ← expression, variables, datasource
        ├── xjasper-renderer-pdf
        ├── xjasper-renderer-image
        └── xjasper-renderer-html
              └── xjasper-engine ← 所有核心 crate
                    ├── xjasper-wasm ← engine
                    └── xjasper-cli ← engine
```

### 2.3 前端 Monorepo

使用 pnpm workspace：

```yaml
# pnpm-workspace.yaml
packages:
  - 'apps/*'
  - 'packages/*'
```

---

## 3. 开发工作流

### 3.1 创建新 Crate

```bash
# 在 crates/ 目录下创建新 crate
cargo new --lib crates/xjasper-new-module

# 在 Cargo.toml 中添加到 workspace
# [workspace]
# members = [
#     ...
#     "crates/xjasper-new-module",
# ]
```

### 3.2 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定 crate 的测试
cargo test -p xjasper-expression

# 运行特定测试
cargo test test_name

# 显示测试输出
cargo test -- --nocapture
```

### 3.3 代码格式化

```bash
# 格式化所有 Rust 代码
cargo fmt

# 检查格式（CI 用）
cargo fmt -- --check

# 格式化前端代码
pnpm format
```

### 3.4 代码检查

```bash
# Clippy 检查
cargo clippy -- -D warnings

# 前端 ESLint 检查
pnpm lint
```

### 3.5 构建

```bash
# Debug 构建
cargo build

# Release 构建
cargo build --release

# 构建 WASM
cd crates/xjasper-wasm
wasm-pack build --target web

# 构建桌面应用
cd apps/desktop
pnpm tauri build
```

---

## 4. 编码规范

### 4.1 Rust 代码规范

#### 命名约定

```rust
// 模块名：snake_case
mod template_parser;

// 类型名：PascalCase
struct Template { }
enum BandType { }
trait DataSource { }

// 函数名：snake_case
fn parse_template() { }

// 常量：SCREAMING_SNAKE_CASE
const MAX_PAGE_SIZE: u32 = 10000;

// 生命周期：小写单字母
fn process<'a>(data: &'a str) { }
```

#### 错误处理

使用 `Result` 和 `thiserror`：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("Expression error: {0}")]
    ExpressionError(String),
}

pub type Result<T> = std::result::Result<T, TemplateError>;
```

#### 文档注释

```rust
/// 解析 JSON 模板为 Template 结构
///
/// # Arguments
///
/// * `json` - JSON 字符串
///
/// # Returns
///
/// 解析成功返回 `Template`，失败返回 `TemplateError`
///
/// # Examples
///
/// ```
/// use xjasper_template::parse_template;
///
/// let json = r#"{"name": "test"}"#;
/// let template = parse_template(json)?;
/// ```
pub fn parse_template(json: &str) -> Result<Template> {
    // ...
}
```

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template() {
        let json = r#"{"name": "test"}"#;
        let result = parse_template(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = "invalid";
        let result = parse_template(json);
        assert!(result.is_err());
    }
}
```

### 4.2 TypeScript 代码规范

#### 命名约定

```typescript
// 接口：PascalCase
interface Template {
  name: string;
}

// 类型别名：PascalCase
type BandType = 'title' | 'detail' | 'summary';

// 函数：camelCase
function parseTemplate(json: string): Template {
  // ...
}

// 常量：SCREAMING_SNAKE_CASE
const MAX_PAGE_SIZE = 10000;

// React 组件：PascalCase
function TemplateEditor() {
  return <div>...</div>;
}
```

#### 类型定义

```typescript
// 优先使用 interface
interface Template {
  name: string;
  version: string;
  bands: Bands;
}

// 联合类型用 type
type ElementType = 'text' | 'image' | 'line';

// 避免 any，使用 unknown
function process(data: unknown) {
  if (typeof data === 'string') {
    // ...
  }
}
```

#### React 组件

```typescript
import React, { useState } from 'react';

interface Props {
  template: Template;
  onSave: (template: Template) => void;
}

export function TemplateEditor({ template, onSave }: Props) {
  const [name, setName] = useState(template.name);

  const handleSave = () => {
    onSave({ ...template, name });
  };

  return (
    <div>
      <input value={name} onChange={(e) => setName(e.target.value)} />
      <button onClick={handleSave}>Save</button>
    </div>
  );
}
```

---

## 5. 测试指南

### 5.1 单元测试

每个 crate 都应该有充分的单元测试。

**测试文件位置：**
- 小型测试：与源码同文件，`#[cfg(test)] mod tests { }`
- 大型测试：`tests/` 目录下

**示例：**

```rust
// src/expression.rs
pub fn evaluate(expr: &str) -> Result<Value> {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let result = evaluate("1 + 2");
        assert_eq!(result.unwrap(), Value::Int(3));
    }

    #[test]
    fn test_field_reference() {
        let result = evaluate("$F{name}");
        // ...
    }
}
```

### 5.2 集成测试

在 `tests/` 目录下编写集成测试。

```rust
// tests/integration_test.rs
use xjasper_engine::ReportEngine;

#[test]
fn test_end_to_end() {
    let engine = ReportEngine::new();
    let template_json = include_str!("../examples/invoice.json");
    let data_json = include_str!("../examples/data.json");

    let template = engine.compile_template(template_json).unwrap();
    let data = engine.load_json_data(data_json).unwrap();
    let filled = engine.fill_report(&template, data, Default::default()).unwrap();
    let pdf = engine.export_pdf(&filled).unwrap();

    assert!(!pdf.is_empty());
}
```

### 5.3 性能测试

使用 `criterion` 进行性能基准测试。

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "layout_benchmark"
harness = false
```

```rust
// benches/layout_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xjasper_layout::LayoutEngine;

fn benchmark_layout(c: &mut Criterion) {
    let engine = LayoutEngine::new();
    let template = load_test_template();
    let data = load_test_data();

    c.bench_function("layout 1000 rows", |b| {
        b.iter(|| {
            engine.layout(black_box(&template), black_box(&data))
        });
    });
}

criterion_group!(benches, benchmark_layout);
criterion_main!(benches);
```

运行基准测试：

```bash
cargo bench
```

### 5.4 WASM 测试

```bash
cd crates/xjasper-wasm
wasm-pack test --headless --firefox
```

---

## 6. 调试技巧

### 6.1 Rust 调试

#### 使用 dbg! 宏

```rust
let result = some_function();
dbg!(&result);  // 打印变量值和位置
```

#### 使用 RUST_LOG 环境变量

```rust
use log::{info, debug, error};

fn process() {
    info!("Processing started");
    debug!("Debug info: {:?}", data);
    error!("Error occurred: {}", err);
}
```

```bash
RUST_LOG=debug cargo run
```

#### 使用 rust-gdb / rust-lldb

```bash
# 构建 debug 版本
cargo build

# 使用 gdb 调试
rust-gdb target/debug/xjasper-cli

# 设置断点
(gdb) break main
(gdb) run
(gdb) next
(gdb) print variable
```

### 6.2 WASM 调试

#### 浏览器 DevTools

```javascript
import init, { compileTemplate } from '@xjasper/core';

await init();

try {
  const result = compileTemplate(json);
  console.log('Success:', result);
} catch (err) {
  console.error('Error:', err);
}
```

#### wasm-pack 测试

```bash
wasm-pack test --headless --firefox -- --nocapture
```

### 6.3 前端调试

#### React DevTools

安装 React DevTools 浏览器扩展，查看组件树和 props。

#### Redux DevTools（如果使用 Redux）

```typescript
import { configureStore } from '@reduxjs/toolkit';

const store = configureStore({
  reducer: rootReducer,
  devTools: process.env.NODE_ENV !== 'production',
});
```

---

## 7. 性能优化

### 7.1 Rust 性能优化

#### 使用 Release 构建

```bash
cargo build --release
```

#### 避免不必要的克隆

```rust
// 不好：每次都克隆
fn process(data: Vec<String>) {
    for item in data.clone() {
        // ...
    }
}

// 好：使用引用
fn process(data: &[String]) {
    for item in data {
        // ...
    }
}
```

#### 使用 Cow 避免不必要的分配

```rust
use std::borrow::Cow;

fn process(s: &str) -> Cow<str> {
    if s.contains("old") {
        Cow::Owned(s.replace("old", "new"))
    } else {
        Cow::Borrowed(s)
    }
}
```

#### 使用 SmallVec 优化小数组

```rust
use smallvec::SmallVec;

// 栈上分配，最多 8 个元素
let mut vec: SmallVec<[u32; 8]> = SmallVec::new();
```

### 7.2 WASM 性能优化

#### 减少 JS ↔ Rust 边界调用

```rust
// 不好：多次调用
for item in items {
    process_item(item);  // 每次都跨边界
}

// 好：批量处理
process_items(items);  // 一次跨边界
```

#### 使用 wasm-opt 优化

```bash
wasm-pack build --release
wasm-opt -Oz -o output.wasm input.wasm
```

---

## 8. 发布流程

### 8.1 版本号管理

遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR**：不兼容的 API 变更
- **MINOR**：向后兼容的功能新增
- **PATCH**：向后兼容的问题修复

### 8.2 发布 Rust Crate

```bash
# 更新版本号
# 编辑 Cargo.toml: version = "0.2.0"

# 构建和测试
cargo build --release
cargo test

# 发布到 crates.io
cargo publish -p xjasper-types
cargo publish -p xjasper-expression
# ... 按依赖顺序发布
```

### 8.3 发布 npm 包

```bash
cd crates/xjasper-wasm

# 构建 WASM
wasm-pack build --release --target web

# 发布到 npm
cd pkg
npm publish
```

### 8.4 发布桌面应用

```bash
cd apps/desktop

# 构建
pnpm tauri build

# 生成的安装包在 src-tauri/target/release/bundle/
```

---

## 9. 常见问题

### 9.1 编译错误

**问题：** `error: linker 'cc' not found`

**解决：**
```bash
# Ubuntu/Debian
sudo apt install build-essential

# macOS
xcode-select --install
```

### 9.2 WASM 构建失败

**问题：** `wasm-pack` 找不到

**解决：**
```bash
cargo install wasm-pack
```

### 9.3 前端依赖安装失败

**问题：** `pnpm install` 报错

**解决：**
```bash
# 清理缓存
pnpm store prune

# 重新安装
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

---

## 10. 资源链接

### 官方文档

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Tauri Docs](https://tauri.app/v2/guides/)

### 相关 Crate

- [rust_decimal](https://docs.rs/rust_decimal/)
- [printpdf](https://docs.rs/printpdf/)
- [rhai](https://rhai.rs/)
- [tiny-skia](https://docs.rs/tiny-skia/)

### 社区

- [Rust 中文社区](https://rustcc.cn/)
- [Rust Discord](https://discord.gg/rust-lang)

---

## 11. 贡献者指南

### 提交 Pull Request 前的检查清单

- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 所有测试通过 `cargo test`
- [ ] 添加了必要的单元测试
- [ ] 更新了相关文档
- [ ] 提交信息遵循 Conventional Commits

### Commit Message 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type：**
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例：**
```
feat(layout): add support for nested groups

Implement nested group support in layout engine.
Groups can now have parent groups, enabling multi-level
hierarchical grouping.

Closes #123
```