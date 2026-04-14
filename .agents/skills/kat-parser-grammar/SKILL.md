---
name: kat-parser-grammar
description: 为 kat-parsers 新增或更新 grammar 支持；当需要引入新的 tree-sitter grammar、同步上游资产、生成 vendor parser crate、统一元数据与命名规范时使用。
---

为 `kat-parsers` 新增或更新单个 grammar 的工作流与约定。

## Quick start

```bash
cd /Users/jun.fan/Code/kat-parsers
sed -n '1,220p' README.md
sed -n '1,260p' xtask/src/main.rs
sed -n '1,220p' crates/tree-sitter-kat-parsers/src/lib.rs
sed -n '1,240p' crates/tree-sitter-kat-parsers/build.rs
sed -n '1,220p' metadata/*.toml
```

以 `<lang>` 表示目标语言的小写目录名，例如 `coffeescript`、`crystal`。

## 什么时候用

- 新增一个新的 tree-sitter grammar 到这个仓库
- 更新某个 grammar 的上游 revision 或快照资产
- 调整 `xtask` / `build.rs` / `src/lib.rs`，让新的 grammar 能被统一生成和导出
- 需要确认本仓库对 grammar 资产的最小化快照边界和命名规范

## 固定流程

1. 先读取仓库基线：
   - [README.md](../../../README.md)
   - [xtask/src/main.rs](../../../xtask/src/main.rs)
   - [crates/tree-sitter-kat-parsers/build.rs](../../../crates/tree-sitter-kat-parsers/build.rs)
   - [crates/tree-sitter-kat-parsers/src/lib.rs](../../../crates/tree-sitter-kat-parsers/src/lib.rs)
   - [metadata](../../../metadata)
   - [THIRD_PARTY.md](../../../THIRD_PARTY.md)
2. 先补一个失败测试，再改实现。
   - 最小测试目标：`tree-sitter-kat-parsers` 能暴露 `<LANG>_LANGUAGE`，并能被 `tree_sitter::Parser` 成功加载。
   - 如果已经暴露 `GRAMMAR` / `NODE_TYPES` 常量，测试里顺手断言非空。
3. 确认上游 grammar 的最小必要资产。
   - 必选：`grammar.js`
   - 常见可选：`tree-sitter.json`、`src/scanner.c`、`src/unicode.c`、`src/tree_sitter/*.h`
   - 先看上游仓库实际文件，不要套用别的语言模板。
4. 将“最小源码快照”复制到 `upstreams/<lang>/`。
   - 只复制生成和构建必需文件。
   - 同步上游许可证文件，文件名保持上游原样，例如 `LICENSE` 或 `LICENSE.txt`。
   - 不把 `parser.c`、`grammar.json`、`node-types.json` 这类生成物放进 `upstreams/`。
5. 新增 `metadata/<lang>.toml`。
   - 用 [metadata-template.toml](assets/metadata-template.toml) 起步。
   - `copied.files` 只列上游快照文件。
   - `generated.outputs` 只列 `vendor/<lang>/` 下实际由本仓库生成或同步的产物。
6. 注册生成与构建逻辑。
   - 在 `xtask/src/main.rs` 的 `grammar_specs()` 中新增 `<lang>`。
   - `generator_root_files` 只填生成阶段需要放到临时根目录的文件，例如 `grammar.js`、可选的 `tree-sitter.json`。
   - `vendored_support_files` 只填从 `upstreams/<lang>/src/` 同步到 `vendor/<lang>/` 的支持文件。
   - 在 `build.rs` 中登记该语言是否有 `scanner.c` / `unicode.c`。
7. 在 `src/lib.rs` 暴露统一接口。
   - `extern "C"` 声明 `tree_sitter_<lang>()`
   - 暴露 `<LANG>_LANGUAGE`
   - 暴露 `<LANG>_GRAMMAR`
   - 暴露 `<LANG>_NODE_TYPES`
8. 生成产物并验证。
   - `cargo run -p xtask -- generate <lang>`
   - 必要时再跑一次 `cargo run -p xtask -- generate`，确保全量流程一致
   - `just check`
9. 更新文档。
   - `THIRD_PARTY.md` 增加上游来源、license、导入资产、生成产物
   - 只有当仓库级流程或规则发生变化时才更新 `README.md`

## 统一规范

- 语言标识统一使用小写 ASCII 名称，贯穿 `upstreams/<lang>`、`metadata/<lang>.toml`、`vendor/<lang>`、`xtask` 的 `name`。
- Rust 暴露常量统一使用全大写前缀：`<LANG>_LANGUAGE`、`<LANG>_GRAMMAR`、`<LANG>_NODE_TYPES`。
- 不手改 `vendor/<lang>/parser.c`、`grammar.json`、`node-types.json`；这些文件只通过 `xtask generate` 生成。
- `build.rs` 只编译已经提交到仓库里的 C 产物，不在构建期执行 `tree-sitter generate`。
- `upstreams/` 只保留最小上游快照，不放 query、测试、bindings 等下游不需要的内容。
- `metadata`、`THIRD_PARTY.md`、`vendor/<lang>/` 必须和上游快照一起更新，避免仓库状态半完成。
- 如上游缺少 `tree-sitter.json`，允许继续接入；`tree-sitter` CLI 可能降到 ABI 14，并打印 warning。默认保持上游原样；如果项目已明确决定在本仓库维护一个本地 overlay 版 `tree-sitter.json`，要把它视为仓库自维护资产，并同步更新 `metadata`、`THIRD_PARTY.md` 与生成 spec。
- 新语言如没有 `scanner.c` 或 `unicode.c`，不要为了对齐别的语言去制造空文件；直接在 spec 里声明 absence。

## 常用命令

```bash
git ls-remote <upstream-repo> HEAD
git clone --depth 1 <upstream-repo> /tmp/<repo-name>
find /tmp/<repo-name> -maxdepth 3 -type f | sort
cargo run -p xtask -- generate <lang>
cargo run -p xtask -- generate
just check
```

## 冷门情况怎么处理

- 上游文件布局不确定时，先枚举文件树，再决定 `generator_root_files` 和 `vendored_support_files`。
- 如果 `tree-sitter generate --no-parser` 能跑通，通常说明 `grammar.js` 足够生成 `grammar.json` / `node-types.json`。
- 如果某语言需要额外根目录输入文件，优先在 `xtask` 里显式声明，不要写死在通用逻辑里。
- 如果修改了 parser input hash 规则，记得接受现有语言 `.parser-inputs` 的更新。

## 完成定义

- 新语言在 `cargo test -p tree-sitter-kat-parsers <test-name>` 下先从失败变为通过
- `cargo run -p xtask -- generate <lang>` 成功
- `just check` 成功
- `git status` 中只剩预期改动

## 资源

- [metadata-template.toml](assets/metadata-template.toml)
- [README.md](../../../README.md)
- [xtask/src/main.rs](../../../xtask/src/main.rs)
- [build.rs](../../../crates/tree-sitter-kat-parsers/build.rs)
- [lib.rs](../../../crates/tree-sitter-kat-parsers/src/lib.rs)
