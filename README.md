# kat-parsers

`kat-parsers` 是 [`kat`](https://github.com/happy-proto/kat) 专用的外部 Tree-sitter parser crate 仓库。

这个仓库只负责两件事：

- 保存从上游 grammar 快照而来的最小源码资产
- 通过 `tree-sitter` CLI 预先生成并提交 `kat` 可直接通过 `git` 依赖使用的 Rust parser crate

它不负责 `kat` 主仓库里的 query、runtime、detector 或语言级集成逻辑。

## 当前布局

- `upstreams/<lang>/`：从上游复制过来的最小源码资产
- `metadata/<lang>.toml`：来源、保留原因与生成约定
- `crates/tree-sitter-kat-parsers/vendor/<lang>/`：生成后并提交到 git 的 parser 产物
- `xtask/`：生成与校验逻辑
- `docs/vendor-grammar-reasons.md`：这些 grammar 过去为什么长期留在 `kat` 主仓库里 vendored 的原因清单

## 使用方式

本地要求：

- Rust toolchain
- `tree-sitter` CLI 在 `PATH` 中
- `just`

常用命令：

```bash
just generate
just check-generated
just check
```

- `just generate`：总是重生成 `grammar.json` / `node-types.json`，总是同步各语言声明的 vendored support files，只在 grammar 输入、临时合成的最小 `tree-sitter.json` 模板或 `tree-sitter` CLI 版本变化时重生成 `parser.c`；生成阶段统一产出 ABI 15 parser
- `just check-generated`：重跑一次生成，然后用 `git diff --exit-code` 确认仓库里提交的 `parser.c` / `grammar.json` 等产物是最新的
- `just check`：执行 `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings` 和 `cargo test --workspace`

`kat` 侧可直接按 git dependency 引用统一 crate：

```toml
tree-sitter-kat-parsers = { git = "https://github.com/happy-proto/kat-parsers", rev = "<commit>", package = "tree-sitter-kat-parsers" }
```

## 维护边界

- 上游 grammar 的来源与 revision 见 `THIRD_PARTY.md`；为什么这些 grammar 被外置到本仓库，见 `docs/vendor-grammar-reasons.md`
- `tree-sitter` CLI 负责预先生成 `grammar.json` / `node-types.json` / `parser.c`
- 仓库不提交 `tree-sitter.json`；如某些 grammar 为了稳定生成 ABI 15 需要它，`xtask` 会只在临时工作目录里合成一份最小配置
- parser 生成发生在 `just` / GitHub Actions 中，而不是 `build.rs`
- `build.rs` 只负责编译已经提交到仓库里的 C / C++ 产物

## TODO

- CoffeeScript 在生成时仍会出现 `unnecessary conflicts` warning；当前不影响生成和使用，先记录为已知情况，后续如开始深度维护该 grammar，再评估是否清理上游 `grammar.js` 中的陈旧 `conflicts` 声明
