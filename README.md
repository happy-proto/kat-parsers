# kat-parsers

[![build](https://github.com/happy-proto/kat-parsers/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/happy-proto/kat-parsers/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/happy-proto/kat-parsers/status.svg)](https://deps.rs/repo/github/happy-proto/kat-parsers)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/happy-proto/kat-parsers/blob/master/LICENSE)

`kat-parsers` 为 [`kat`](https://github.com/happy-proto/kat)、[sidediff](https://github.com/happy-proto/sidediff) 和其它 Tree-sitter 消费者提供统一的预生成 parser、压缩状态表与共享解压运行时。

本仓库集中维护语法输入、scanner、许可证及来源锁定，生成并提交可直接编译的产物，同时提供批量上游同步和消费者 Git 依赖更新工作流。解析仍由 Tree-sitter 执行；高亮 query、语言检测和应用集成由消费者维护。

## 消费者接入

通过 Git 依赖使用聚合 crate：

```toml
[dependencies]
tree-sitter-kat-parsers = { git = "https://github.com/happy-proto/kat-parsers" }
```

普通消费者只需 Rust 和 C/C++ 编译环境。生成的 C、JSON 和共享头文件已提交到 Git，各语言的 `build.rs` 直接编译这些产物；消费方不需要 Node.js、Tree-sitter CLI 或额外的压缩构建步骤。

压缩的是状态表，parser 函数和 scanner 仍正常编译进二进制。生成阶段仅在有体积收益时采用 Zstd；对应语言首次加载时，通过 `kat-parser-runtime` 和每语言的 `OnceLock` 解压一次，恢复的表保留到进程退出，后续解析直接使用原生表。

具体 Git revision 由最终应用提交的 `Cargo.lock` 固定，依赖库自己的 lockfile 不会被下游继承。需要自动跟踪更新时，不要在依赖声明中设置 `rev`；消费方可接入 [Git 依赖同步工作流](docs/git-dependencies.md)，统一更新直接与间接 Git 锁定。

## 维护与生成

仓库维护需要 Rust、C/C++ 编译环境、Git 和 `just`。`just check`、`just test` 与来源检查不需要 Node.js 或 Tree-sitter CLI；涉及重新生成时，额外需要 Node.js 24 和 `tree-sitter` CLI 0.27.1（与 Generated Check 一致）：

```bash
npm install -g tree-sitter-cli@0.27.1
```

| 命令 | 用途 |
| --- | --- |
| `just check-upstreams` | 检查所有远程来源是否有更新；发现更新时返回非零，不写工作区 |
| `just check-upstream-sources` | 按锁定 revision 重放来源和补丁，验证已保存输入没有漂移 |
| `just sync` | 批量更新来源和锁定 revision，重新生成产物并运行仓库检查 |
| `just generate` | 从当前保存的输入增量生成、校验并压缩产物，不拉取上游更新 |
| `just check-generated` | 增量生成后执行 `git diff --exit-code`，需从干净工作区运行 |
| `just check` | 执行格式、Clippy 和普通 workspace 测试 |
| `just test` | 仅运行普通 workspace 测试 |

来源同步会更新声明的快照，请从干净工作区开始；需要保留的本地语义、路径或安全修复应记录为最小补丁，不能只修改快照文件。具体约定见 [批量维护上游语法](docs/upstreams.md)。

生成流程使用官方预编译 CLI 和 ABI 15；`xtask` 负责输入准备、缓存、状态表转换与压缩。相同内容保留原修改时间，避免触发无效重编译。源码、补丁和生成参数变化后的产物需要一并提交。缓存和运行时边界见 [生成与构建](docs/generation.md)。

## CI 与更新机制

普通 CI 在 Linux、macOS 和 Windows 上运行格式、Clippy 和 workspace 测试。独立的 Generated Check 会重放锁定来源，运行生成契约测试，再执行 `cargo run -p xtask -- generate --force` 和 `git diff --exit-code`；它强制重新生成全部产物，与本地增量检查互补。

Ghostty 官方解析器对照由独立 macOS CI 执行。普通测试不下载或运行 Ghostty；需要本地对照时使用 `GHOSTTY_BIN=/path/to/ghostty just ghostty-check`，详见 [Ghostty grammar 与验证](docs/ghostty.md)。

| 更新对象 | 维护入口 |
| --- | --- |
| 本仓库的 Cargo 依赖和 GitHub Actions | [Dependabot](.github/dependabot.yml) |
| 上游 grammar、scanner 和许可证 | [每周同步工作流](.github/workflows/sync-upstreams.yml)，来源与补丁由 `metadata/` 声明 |
| 消费者的直接、间接 Git 锁定 | [可复用工作流](.github/workflows/sync-git-dependencies.yml)，由消费者配置定时或手动调用 |

语法同步与消费者 Git 依赖更新会推送更新分支并显式调度 CI。组织策略禁止 Actions 创建 PR，因此工作流在执行摘要中提供 Compare 入口，由维护者创建和审阅更新 PR。上游源码与生成产物合入后，消费者再更新自己的锁文件。

## 仓库布局与审查

- `upstreams/<lang>/`、`metadata/<lang>.toml`、`patches/`：语法输入、来源 revision、资产映射与最小必要补丁；自维护语言在 metadata 中明确标记。
- `crates/tree-sitter-kat-<lang>/`：各语言的 Rust 入口、生成的 C/JSON，以及 scanner 等构建资产。
- `crates/tree-sitter-kat-parsers/`：聚合入口，供消费者通过单一依赖面使用托管语言。
- `crates/kat-parser-common/`：共享 C/C++ 构建支撑与 Tree-sitter 头文件。
- `crates/kat-parser-runtime/`：共享状态表解压实现，由语言入口自动调用。
- `crates/parser-conformance/`、`crates/ghostty-conformance/`：解析对照测试，不属于消费者依赖链路。
- `xtask/`：来源同步、生成、压缩与校验逻辑。

生成的 `parser.c`、`grammar.json`、`node-types.json` 和缓存摘要使用 `linguist-generated` 标记，GitHub 默认折叠其 diff，审查时仍可展开。上游 grammar、scanner 和补丁保持正常展示；第三方源码保留上游格式与模块结构。

来源与许可证见 [THIRD_PARTY.md](THIRD_PARTY.md) 和 `metadata/`；语言迁移的历史原因见 [grammar 维护理由](docs/vendor-grammar-reasons.md)。
