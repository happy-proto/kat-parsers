# 生成与构建

## 生成器边界

`xtask` 负责输入准备、缓存、产物同步和 Rust crate scaffolding；Tree-sitter CLI 负责 grammar 求值和 parser 生成。我们使用预编译且经过优化的 CLI，不将生成库嵌入 `xtask`。

将生成器编进 `xtask` 会让两种速度目标耦合：低优化等级可以缩短本地或 CI 的编译时间，却牺牲生成器执行效率；高优化等级可以加快生成，却增加生成器本身的编译成本。预编译 CLI 将这两个成本分开，`xtask` 可以快速构建，生成器也可以保持优化后的执行速度。因此 Generated Check 使用 npm 安装官方预编译 CLI，而不是 `cargo install` 或通过 Rust 依赖构建生成器。

普通 `tree-sitter` Rust 库用于解析和加载测试，不负责从 `grammar.js` 生成 C parser。这种 runtime 依赖与生成器是两个不同的职责。

## Tree-sitter 0.27 的生成契约

- 本地和 CI 使用同一固定版本 CLI，生成参数固定为 `--abi 15 --js-runtime node`，保留生成器默认的解析状态优化。
- 不合成 `tree-sitter.json`。0.27 可以在没有配置文件时生成 ABI 15，语言版本元数据采用默认 `0.0.0`。这个版本是 grammar 元数据，不是 Tree-sitter runtime 版本。
- 每个需要更新的 grammar 只调用一次 `generate`，同时产生 `grammar.json`、`node-types.json` 和 `parser.c`。
- JavaScript runtime 是执行 grammar 源码的程序。统一选择 Node.js，因为 Less/SCSS 复用的 CSS grammar 使用 ES module；CLI 内置 QuickJS 的 `require` loader 无法加载该模块。不增加逐语言 runtime 选择或自动降级，也不为使用 QuickJS 改写上游 grammar。
- Node.js 和 CLI 只参与维护时生成。消费方仍通过 `build.rs` 编译已提交的 C/C++，不需要安装它们。

## 缓存和构建失效

`.parser-inputs` 保存两行 BLAKE3 摘要：完整生成输入，以及三个生成产物和共享头文件。输入包括 CLI 版本、ABI、JS runtime、所有声明的 JS/JSON 文件，以及跨目录的 `generator_support_files`。文件路径也参与摘要，避免不同模块布局意外复用缓存。

只有输入和产物都匹配时才跳过生成。产物缺失或被编辑也会触发重生成；`--force` 无条件重新生成全部产物，供 CI 独立验证确定性。不存在旧缓存格式兼容层。

scanner 等 support files 独立同步：修改 scanner 不需要重算 parser 状态表，但仍必须进入 vendored crate。`parser.h`、`alloc.h` 和 `array.h` 统一同步自 CLI 的生成结果，不复制 grammar 私有的旧版本，也不使用 runtime 内部头文件。新 grammar 的所有 JS/JSON import 都必须在 metadata 中声明；不依赖临时目录外的 npm 包或机器上的隐式文件。

内容相同的文件保持原修改时间。Rust 模板先在临时目录中格式化，再与 checkout 比较，避免“写入未格式化模板、然后 rustfmt 改回”的无效重编译。C/C++ 构建只监听源文件和头文件，并包含共享头文件；grammar JSON 和生成缓存变化不会单独触发 C 重编译。

## 验证

普通 `just check` 执行格式、Clippy 和 workspace 测试，包括 Crystal 插值/heredoc 与 dotenv assignment 边界。需要外部 CLI 的生成契约测试集中在 Generated Check，运行 `cargo test -p xtask -- --include-ignored`，覆盖跨目录 import、scanner 独立同步、缺失或损坏产物和共享头文件恢复、无配置 ABI 15、优化后的 C/C++ array 操作，以及相同产物的修改时间保持。

Generated Check 随后强制重生成所有 grammar，再执行 `git diff --exit-code`。这个独立检查保留，即使本地使用缓存，也不会把缓存当作生成正确性的证明。

性能对照应使用已构建的 `xtask` 二进制，分别测量 `generate --force` 和输入未变的 `generate`。消费方构建则应区分首次编译、源码实际变化后的编译，以及重生成但产物不变后的增量构建。mbx/Cargo 缓存命中不等同于首次编译速度；生成器时间也不等同于 kat 的解析性能。

2026-10-13 的本地对照：Apple M1 Pro、macOS arm64、Node.js 26.11.1、Tree-sitter CLI 0.27.1。基线是 `37f1943be08fdc1068b5cd5997f2e311e52cc4c4`，两组使用已构建的 debug `xtask` 和同一个预编译 CLI；用 `/usr/bin/time -p` 测量进程启动至退出，包括 JS 求值、生成、文件同步和 Rust 格式化，不包含 `xtask` 编译。以下是单次观测，不作为跨机器性能保证。

| 工作负载 | 基线 | 本次适配 |
| --- | ---: | ---: |
| 73 个 grammar，强制生成 | 41.98 秒 | 33.17 秒 |
| 73 个 grammar，输入和产物未变 | 20.65 秒 | 4.23 秒 |

本次适配中，相同产物强制重生成后，`cargo check -p tree-sitter-kat-parsers` 耗时 0.21 秒，未出现 `Compiling` 或 `Checking`，说明生成没有触发 C/Rust 重编译。这是已有构建产物的增量检查，不是首次构建测量。普通 workspace 测试与 Node.js 24 的生成契约测试均通过；尚未测量 kat 端到端解析吞吐量。

## 预生成压缩表

所有托管 grammar 经过同一条生成流程。`xtask` 在 CLI 生成 C 之后，编译一个只读取静态数组的 C 导出工具，把前两个状态以外的行转换成 Tree-sitter 原生紧凑表，再用 Zstd level 19 压缩；仅在压缩数据加保留的两行小于原表时采用压缩，不设置语言体积门槛。导出工具逐项比较全部状态与符号的原表值和紧凑表值，转换错误会终止生成。parser 的其它数组、scanner、Tree-sitter ABI 和查表逻辑保持原样。

压缩字节直接进入提交的 C 产物，不在消费方重新生成。消费方只需正常构建 Rust 与 C/C++，不需要 Node、Tree-sitter CLI、Python、独立 Cargo home 或应用导出的解压函数。`kat-parser-common` 仍只提供构建支撑；普通依赖 `kat-parser-runtime` 提供共享 Zstd 解码，每个语言的 Rust `LANGUAGE` 包装通过 `OnceLock` 只初始化一次，单独依赖某个语言 crate 也能工作。

文件布局固定为小端 uint16 紧凑表、最多三个补齐字节、四字节对齐的小端 uint32 索引。运行时分配 `Box<[u32]>` 保证对齐，在大端目标分别转换两种整数。解压内存与语言指针均保留至进程结束；首次获取语言有解压成本，后续解析直接使用原生表。压缩减少 binary 体积，并不表示运行时表也只占压缩后的大小。

缓存包含打包器实现、导出工具、格式版本、压缩参数和 Zstd 版本。Generated Check 强制重生成并检查差异，避免只验证缓存。`parser-conformance` 仅用于测试：从提交的压缩数据恢复静态原生表，对照 CoffeeScript scanner、错误输入、增量解析与 lookahead，并验证并发首次初始化；其原生对照表不进入消费方依赖链路。
