---
name: kat-parser-grammar
description: 为 kat-parsers 新增或更新单个 tree-sitter grammar 支持；当需要引入新 grammar、同步上游快照、生成 vendor parser 产物、调整 xtask/build.rs/lib.rs、补 metadata 或 THIRD_PARTY 记录时使用。
---

用于在 `kat-parsers` 中新增或更新单个 grammar。以 `<lang>` 表示语言目录名，例如 `coffeescript`、`crystal`。

## 流程

1. 先确认仓库边界。
   - 阅读 [README.md](../../../README.md)，确认这次改动属于 `kat-parsers` 的职责范围。
   - 修改生成或构建链路时读取 [生成与构建](../../../docs/generation.md)，沿用预编译 CLI、无配置 ABI 15 和完整输入/输出缓存的约定。
2. 先补失败测试，再改实现。
   - 最小目标：`tree-sitter-kat-parsers` 能暴露 `<LANG>_LANGUAGE`，并能被 `tree_sitter::Parser` 成功加载。
   - 如果该语言已经暴露 `GRAMMAR` / `NODE_TYPES` 常量，测试里顺手断言非空。
   - 如果当前任务明确不需要补测试，要在改动说明里写清楚理由。
3. 盘点上游最小必要资产。
   - 必选：`grammar.js`
   - 常见可选：`src/scanner.c`、`src/unicode.c`、`src/tree_sitter/*.h`；生成不需要 `tree-sitter.json`
   - 先看上游仓库实际文件，不要机械套用别的语言模板。
   - 同时确认上游 license、分发条件和附带文件要求；只有在确认当前引用、复制、提交生成产物的方式不违反授权协议后，才继续引入。
   - 可用命令：

```bash
git ls-remote <upstream-repo> HEAD
git clone --depth 1 <upstream-repo> /tmp/<repo-name>
find /tmp/<repo-name> -maxdepth 3 -type f | sort
```

4. 同步上游快照到 `upstreams/<lang>/`。
   - 只复制生成和构建必需文件。
   - 同步上游许可证文件，文件名保持上游原样，例如 `LICENSE` 或 `LICENSE.txt`。
   - 不把 `parser.c`、`grammar.json`、`node-types.json` 这类生成物放进 `upstreams/`。
5. 补 `metadata/<lang>.toml`。
   - 用 [metadata-template.toml](assets/metadata-template.toml) 起步。
   - 参考 [metadata](../../../metadata) 里的现有语言条目。
   - `copied.files` 只列上游快照文件。
   - `generated.outputs` 只列 `vendor/<lang>/` 下实际由本仓库生成或同步的产物。
6. 接生成链路和编译链路。
   - `grammar_specs()` 从 metadata 自动发现语言；JS/JSON import 必须列入 `copied.files`，跨语言目录的 import 列入 `generator_support_files`，使缓存覆盖完整输入。
   - C/C++ scanner 与 support files 由 metadata 同步到各语言 vendor 目录，并由共享 build support 编译；不手改生成的 crate scaffolding。
   - 如果需要新的根目录输入文件，优先在 `xtask` 的 language spec 里显式声明，不要写死到通用逻辑里。
7. 接 crate 导出接口。
   - 生成器自动生成单语言 crate 和聚合 re-export，统一暴露：
     - `<LANG>_LANGUAGE`
     - `<LANG>_GRAMMAR`
     - `<LANG>_NODE_TYPES`
8. 生成并验证。

```bash
cargo run -p xtask -- generate <lang>
cargo run -p xtask -- generate
just check
```

   - 第一条验证单语言接入。
   - 第二条验证全量生成链路一致。
   - 如果修改了 parser input hash 规则，记得接受现有语言 `.parser-inputs` 的更新。
9. 更新记录。
   - 修改第三方资产边界时，更新 [THIRD_PARTY.md](../../../THIRD_PARTY.md)，补充上游来源、license、导入资产、生成产物。
   - 仓库级流程或规则变化时，再更新 [README.md](../../../README.md)。

## 规范

- 语言标识统一使用小写 ASCII 名称，贯穿 `upstreams/<lang>`、`metadata/<lang>.toml`、`vendor/<lang>`、`xtask` 的 `name`。
- Rust 常量统一使用全大写前缀：`<LANG>_LANGUAGE`、`<LANG>_GRAMMAR`、`<LANG>_NODE_TYPES`。
- 不手改 `vendor/<lang>/parser.c`、`grammar.json`、`node-types.json`；这些文件只通过 `xtask generate` 生成。
- `build.rs` 只编译已经提交到仓库里的 C 产物，不在构建期执行 `tree-sitter generate`。
- `upstreams/` 只保留最小上游快照，不放 query、测试、bindings 等下游不需要的内容。
- `tree_sitter/parser.h`、`alloc.h` 和 `array.h` 使用 CLI 生成并共享的版本，不引入 grammar 自带副本；其它 scanner 专用头文件按 metadata 声明。
- `metadata`、`THIRD_PARTY.md`、`vendor/<lang>/` 必须和上游快照一起更新，避免仓库状态半完成。
- 引用第三方 grammar 时，先确认上游协议允许当前这种复制、生成和随仓库分发方式；不要只因为技术上可行就直接 vendoring。
- 新语言如没有 `scanner.c` 或 `unicode.c`，不要制造空文件，直接在 spec 里声明 absence。

## 例外

- 使用项目固定的 Tree-sitter 0.27 CLI 和 ABI 15；缺少 `tree-sitter.json` 时采用原生默认元数据，不合成配置或增加 ABI 14 分支。
- 如生成时出现 `unnecessary conflicts` warning，先区分它是否只是陈旧 `conflicts` 声明；若不影响生成和使用，可以先记录为已知情况，不要顺手清理 grammar。
- 完整生成使用一次 CLI 调用产出三个文件，不另加 `--no-parser` 预生成步骤。

## 完成定义

- 失败测试先被稳定复现，再被实现修复通过；如果本次明确不补测试，说明理由。
- `cargo run -p xtask -- generate <lang>` 成功。
- `cargo run -p xtask -- generate` 成功。
- `just check` 成功。
- `git status` 中只剩预期改动。

## 资源

- [metadata-template.toml](assets/metadata-template.toml)
