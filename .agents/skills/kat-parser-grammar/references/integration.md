# 仓库接入

新增语言、更新快照或改变生成／编译输入时读取。先按 [README.md](../../../../README.md) 确认当前目录布局；改生成或构建链路时读取 [生成与构建](../../../../docs/generation.md) 并核对 `xtask` 实现。以下以 `<lang>` 表示语言标识，crate 名称及常量前缀沿用生成器的转换规则。

## 来源与最小资产

- 先确认上游实际文件、固定 revision 和许可证允许的复制、修改及分发方式，再导入。保留原名的许可证和要求附带的声明；依据规范独立实现时，区分参考实现与复制的代码，不把参考来源写成导入资产。
- `upstreams/<lang>/` 保存生成和编译必需的源码，包括 `grammar.js`、其导入的 JS/JSON、实际需要的 scanner 和专用头文件／support sources。保留本地修补并说明原因；更新上游前先比对，避免覆盖已修复行为。
- 不把上游 `parser.c`、`grammar.json`、`node-types.json` 放进源码快照，不整批复制 query、bindings 或测试语料。持久化回归放在非生成测试路径，与最小源码快照分开。
- 没有 scanner 或其它可选资产就不声明、不制造空文件。scanner 的包含路径和额外 C/C++ 文件按上游实际依赖盘点，不假定固定文件名或布局。

## Metadata 与产物

从 [metadata-template.toml](../assets/metadata-template.toml) 起步，参考 [现有条目](../../../../metadata)；自维护 grammar 记录权威参考与自维护原因，不虚构上游快照。保持 metadata 和 [THIRD_PARTY.md](../../../../THIRD_PARTY.md) 的来源、许可证、本地修改及资产清单一致。

| 内容 | 位置与约束 |
| --- | --- |
| 语言注册 | `metadata/<lang>.toml`；`xtask` 自动发现，无需手工新增语言枚举 |
| 源码输入 | `[copied].files` 列出 `upstreams/<lang>/` 中保留的输入，包含生成必需的全部 JS/JSON import 和 scanner/support files |
| 跨目录生成依赖 | `generator_support_files` 以语言源码目录为基准声明，保留相对布局并纳入输入哈希；不能依赖机器上的隐式 npm 包或临时文件 |
| 产物记录 | `[generated].outputs` 列出实际生成或同步到语言 vendor 目录的文件；这份记录不能代替输入声明 |
| 语言 crate | `crates/tree-sitter-kat-<crate-suffix>/`，其中 `vendor/<lang>/` 存放已提交的 parser 和 support files |
| 共享头文件 | `crates/kat-parser-common/include/tree_sitter/`，使用 CLI 生成的 `parser.h`、`alloc.h`、`array.h`，不复制上游历史副本 |
| 导出 | 单语言 crate 的 `LANGUAGE`／`GRAMMAR`／`NODE_TYPES`，聚合 crate 中对应的 `<LANG>_*`，均由生成器维护 |

`src/` 下的 support files 同步到 vendor 时会去掉该前缀；确认同步后的 include 路径和构建入口有效。当前共享构建逻辑自动编译 vendor 根目录的 C/C++ 翻译单元；遇到其它布局，先检查现有机制能否承载，再针对缺口修改生成／共享构建逻辑及契约测试，不手改单语言 `build.rs` 绕过。

## 生成与构建约束

- 使用仓库与 CI 固定的预编译 Tree-sitter CLI 和 Node.js 版本要求，以 [生成与构建](../../../../docs/generation.md) 和 workflow 为准；当前契约为 CLI 0.27、ABI 15、`--js-runtime node`。不合成 `tree-sitter.json`、不增加 ABI 降级分支。
- 通过 `cargo run -p xtask -- generate <lang>` 生成。一次 CLI 调用同时得到三个 parser 产物，不增加 `--no-parser` 预生成阶段；不手改 vendor 内的 parser、JSON、缓存戳或同步的 scanner。
- `build.rs` 只编译已提交的 C/C++，不运行生成器。Rust scaffolding、聚合 re-export 和共享 build support 由 `xtask` 模板生成；改变行为时修改权威模板，不只改会被覆盖的输出。
- `.parser-inputs` 同时校验完整生成输入与生成输出；scanner/support files 独立同步，即使命中 parser 缓存也要检查它们的实际 vendor 内容。修改哈希规则时复查并接受对应缓存戳的预期更新。
- 自定义测试放在语言 crate 的 `tests/` 或独立测试 crate，避免写进生成的 `src/lib.rs`。需要额外测试依赖时考虑 manifest 也由模板维护，不能依赖下次生成就会消失的手工修改。
- 遇到生成 warning，区分陈旧声明、生成失败和实际解析缺陷。无行为影响的 warning 可以记录；不能仅为消除提示顺手改变语法，也不能用 warning 掩盖未通过的生成或测试。

## 最后核对

单语言生成后检查目标 crate 与聚合导出；全量生成后检查影响范围。快照、metadata、归属记录和 vendor 一起交付，确认新增依赖及锁文件的变化符合项目工具链。仓库级维护契约变化才同步更新 README 或生成文档，不把每次语言修复过程写进仓库说明。
