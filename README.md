# kat-parsers

`kat-parsers` 是 [`kat`](https://github.com/happy-proto/kat) 专用的外部 Tree-sitter parser crate 仓库。

这个仓库只负责两件事：

- 保存从上游 grammar 快照而来的最小源码资产
- 生成并提交 `kat` 可直接通过 `git` 依赖使用的 Rust parser crate

它不负责 `kat` 主仓库里的 query、runtime、detector 或语言级集成逻辑。

## 当前布局

- `upstreams/<lang>/`：从上游复制过来的最小源码资产
- `metadata/<lang>.toml`：上游来源、revision 与生成约定
- `crates/tree-sitter-kat-parsers/vendor/<lang>/`：生成后并提交到 git 的 parser 产物
- `xtask/`：生成与校验逻辑

## 使用方式

本地要求：

- Rust toolchain
- `tree-sitter` CLI 在 `PATH` 中
- `just`

常用命令：

```bash
just generate
just check
```

- `just generate`：总是重生成 `grammar.json` / `node-types.json`，总是同步各语言声明的 vendored support files，只在 `grammar.json`、可选的 `tree-sitter.json` 或 `tree-sitter` CLI 版本变化时重生成 `parser.c`
- `just check`：执行 `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings` 和 `cargo test --workspace`

`kat` 侧可直接按 git dependency 引用统一 crate：

```toml
tree-sitter-kat-parsers = { git = "https://github.com/happy-proto/kat-parsers", rev = "<commit>", package = "tree-sitter-kat-parsers" }
```

## 维护边界

- 上游 grammar 的来源与 revision 见 `metadata/` 和 `THIRD_PARTY.md`
- parser 生成发生在 `just`/CI 中，而不是 `build.rs`
- `build.rs` 只负责编译已经提交到仓库里的 C 产物
