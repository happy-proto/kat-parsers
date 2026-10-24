# 从 kat 外置到 kat-parsers 的 grammar 清单

本文档记录这次从 `kat` 外置到 `kat-parsers` 的 grammar 来源，以及它们此前为什么长期留在 `kat` 主仓库里作为 vendored grammar。

现在全部 parser 源统一维护在 `kat-parsers` 的独立语言 crate，由本仓库生成紧凑表和 Zstd 产物，并提供共享解压运行时。下列分类保留历史引入背景；当前不再按大小或公开 crate 是否可用决定分散依赖来源。批量同步、固定 revision 和本地补丁的约定见 [批量维护上游语法](upstreams.md)。

## 上游 parser crate 迁入统一生成仓库

- `ada`、`bash`、`batch`、`c`、`c_sharp`、`cel`、`cfml`、`clojure`、`cmake`、`cpp`、`css`、`d`、`dart`、`diff`、`elixir`、`elm`、`embedded_template`、`erlang`、`fortran`、`fsharp`、`go`、`groovy`、`html`、`ini`、`java`、`javascript`、`jsdoc`、`json`、`kotlin`、`lua`、`make`、`systemverilog`、`verilog`：此前由 Kat 直接依赖上游 parser crate；现在保留固定上游 revision 的 grammar 输入，由 `kat-parsers` 统一生成并通过聚合 crate 提供 parser。具体来源和许可证见 [`../THIRD_PARTY.md`](../THIRD_PARTY.md) 及对应 `metadata/*.toml`。
- `fsharp_signature`：与 `fsharp` 一起从同一上游 crate 迁入。signature grammar 的共享 F# grammar 作为单独输入同步；两个 scanner 的 include 路径通过仓库内 patch 适配 vendor 目录布局。
- `css`：继续为 LESS/SCSS 提供生成期 grammar support，同时记录上游 crate 来源；scanner 的参数消音改动由可重放 patch 保留。
- `diff`：crate 原 manifest 指向的 GitHub 仓库已删除，来源改从保留同一 revision 的 `tree-sitter-grammars/tree-sitter-diff` 获取。
- `groovy`：上游 revision 没有独立 LICENSE 文件；保留 Cargo manifest 的 MIT 声明作为许可证来源记录。

## 没有可用的 crates.io grammar crate

- `csh`：现有社区 tcsh grammar 仅覆盖占位词与字符串，无法用于真实激活脚本。依据 tcsh 官方手册独立维护 MIT grammar，提供 C shell/tcsh 源码高亮；覆盖边界见 [`../metadata/csh.toml`](../metadata/csh.toml) 与 [`../THIRD_PARTY.md`](../THIRD_PARTY.md)。

- `graph_easy`：没有可直接使用的 parser，依据 Graph::Easy 常见 DSL 独立编写 MIT grammar。与 DOT 分离，支持节点、边标签、属性及分组等源码结构；行为参考与边界见 [`../metadata/graph_easy.toml`](../metadata/graph_easy.toml)。

- `git_config`
- `git_link`
- `git_log`
- `git_mailmap`
- `textproto`
- `userscript_metadata`
- `cabal`
- `crontab`
- `dotenv`
- `email`
- `ninja`
- `twig`
- `ssh_config`
- `git_commit`
- `git_rebase`
- `requirements`
- `apache`
- `sass`
- `todotxt`

## 上游 crate 的依赖约束与当前 runtime 不兼容

- `mermaid`：Merman 发布的 `tree-sitter-mermaid` 0.3.0 固定依赖 `tree-sitter-language =0.1.8`，与 `kat` 的 Tree-sitter 0.27.1 / `tree-sitter-language` 0.1.9 冲突。这里保留其模块化 grammar 与 scanner 的原始快照，通过现有生成流程导出兼容的 `LanguageFn`，不降级 Kat 的 runtime，也不在应用构建时生成 parser。上游 revision、许可证与生成约定见 [`../metadata/mermaid.toml`](../metadata/mermaid.toml)。上游 crate 解除版本约束后可重新评估直接使用公开 crate。

## 已有 crate，但仍依赖旧版 `tree-sitter`，暂时无法和当前 runtime共存

- `query`
- `fish`
- `markdown`
- `markdown_inline`
- `gitattributes`
- `dot`
- `svelte`
- `just`
- `wgsl`

## 当前继续 vendored，主要是为了沿用上游 grammar 源和仓库内 query 集成资产

- `actionscript`
- `applescript`
- `asciidoc`
- `asm`
- `awk`
- `bibtex`
- `csv`
- `latex`
- `nasm`
- `sml`
- `strace`
- `tcl`
- `tsv`
- `typst`
- `vimhelp`

## 仓库内有本地 grammar 或 scanner 改造，短期内不适合直接切回上游 crate

- `ignore`
  原因：本地重命名并调整成统一的 ignore-pattern runtime，供 `.gitignore`、`.dockerignore`、`.npmignore` 等文件共享。
- `authorized_keys`
  原因：当前直接在仓库内维护了一个面向 SSH 公钥/授权文件的小 grammar，并配合 `.pub` 内容检测做路径识别。
- `known_hosts`
  原因：仓库内维护的 OpenSSH 主机公钥记录 grammar，独立表达主机列表、哈希主机、CA/撤销 marker 和公钥字段；语法结构与 `authorized_keys` 不同。
- `cmakecache`
  原因：仓库内维护了一个专门面向 `CMakeCache.txt` 的小 grammar，用来稳定承接 key/type/value 结构和 path-like value 高亮。
- `command_help`
  原因：当前直接在仓库内维护了一个针对命令帮助文本的小 grammar，用来识别标题、directive 和常见命令行片段。
- `cpuinfo`
  原因：仓库内维护了面向 `cpuinfo` 键值结构的本地 mini-runtime，当前没有值得迁移的现成 crate。
- `dockerfile`
  原因：本地维护了适配 `kat` 宿主模型的 injections query，且当前可用 crate 仍停留在旧版 `tree-sitter` 依赖链上。
- `debsources`
  原因：仓库内维护了 `sources.list` 专用 mini-runtime，用来稳定识别源类型、suite/component 和 URI。
- `fortran_namelist`
  原因：当前直接在仓库内维护了一份面向 namelist 结构的轻量 grammar，和主 `fortran` runtime 分开演进更简单。
- `fstab`
  原因：仓库内维护了 `fstab` / `crypttab` / `mtab` 这类系统表文件的小 grammar，便于按字段类型持续细化。
- `hcl`
  原因：本地 grammar/scanner 做过适配，用来承接 `.hcl` 和 `.nomad` 这类文件的独立 runtime。
- `graphql`
  原因：当前 highlights query 是按 vendored grammar 的节点结构维护的本地集成版本。
- `proto`
  原因：当前 runtime 依赖 vendored grammar 的节点结构和现有 query 约定。
- `gomod`
  原因：当前沿用 vendored grammar 资产和本地 query 集成。
- `gowork`
  原因：当前沿用 vendored grammar 资产和本地 query 集成。
- `gosum`
  原因：当前沿用 vendored grammar 资产和本地 query 集成。
- `less`
  原因：本地 grammar 改成复用仓库里的 `css/grammar.js`，避免引入第二套 CSS 依赖。
- `jinja`
  原因：`grammar.js` 是仓库内包装层，指向 vendored base grammar 和 kat 的 HTML 注入模型。
- `vue`
  原因：本地 scanner 改了 HTML scanner 符号名，确保 Vue runtime 和独立 HTML runtime 可同时链接。
- `scss`
  原因：本地 grammar 改成复用仓库里的 `css/grammar.js`，避免引入第二套 CSS 依赖。
- `textile`
  原因：当前仓库内维护的是本地最小 Textile grammar；已评估过上游实现，但其授权元数据暂不适合直接 vendored 进仓库。
- `stylus`
  原因：当前仓库内维护的是面向缩进式 Stylus 语法的本地最小 grammar，优先保证 `.styl` / `.stylus` 的稳定可读性，而不是强行依赖一条维护状态不稳定的外部依赖链。
- `syslog`
  原因：当前直接在仓库内维护了面向经典 syslog 行结构的小 grammar，用来稳定承接月份、时间、主机、进程和消息体的最小可读分层。
- `vyper`
  原因：上游 grammar 在当前集成目标下不够稳定，因此仓库内先维护一份面向 Vyper 合约结构的本地最小 grammar 与 query，优先保证 detector、fixture 和基础高亮可持续演进。

## 许可证原因，继续使用 vendored 方案更合适

- `jq`
  原因：当前 crates.io 上常用的 `tree-sitter-jq` 走 GPL 许可证链路；仓库里改用 BSD 3-Clause 上游并保留 vendored parser 源。

## CSS 的共享生成输入

- `css`
  CSS parser 和 Less/SCSS 共同使用 `upstreams/css/grammar.js`。共享输入在 metadata 中明确声明，统一随 CSS 来源更新，不额外复制第二套 grammar。

## 后续迁移原则

- 新增语言使用独立 parser crate 和统一生成、压缩、解压流程；上游来源、精确 revision、许可证及本地适配必须可重放。
- 上游源码更新集中通过批量同步流程提出；kat 和其它应用仅更新统一 Git 依赖，不另维护压缩或语法源码同步脚本。

## Jam build language

`jam` is independently maintained from the B2 language specification: the published `tree-sitter-jam` crate parses a different programming language. The parser provides structural source highlighting for Jam/B2 build scripts, with raw balanced actions and no build execution or shell assumptions. Reference revision and files are recorded in `metadata/jam.toml` and `THIRD_PARTY.md`.
