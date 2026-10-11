# Ghostty grammar 与验证

Ghostty grammar 由本仓库独立维护，依据官方 [LineIterator](https://github.com/ghostty-org/ghostty/blob/b1d9492cfa5d0d22c534a00e81bea623957113e7/src/cli/args.zig) 与 [Config.loadReader](https://github.com/ghostty-org/ghostty/blob/b1d9492cfa5d0d22c534a00e81bea623957113e7/src/config/Config.zig) 设计，没有复制或改编社区 `tree-sitter-ghostty` 的 grammar、scanner 或 query。参考 revision 与源文件见 [metadata](../metadata/ghostty.toml)。

## 解析契约

- 配置逐行解析；LF 是行边界，CRLF、缩进、空白行和末行无换行均有效；只删除文件开头一个 UTF-8 BOM，后续 BOM 属于数据。
- 去除行首空白后以 `#` 开头的行是注释。值中的 `#`、`;`、`=` 都是数据，不是行尾注释或额外配置项。
- 首个 `=` 分隔 key/value，空值保留为无 value 的 setting；没有 `=` 的裸配置项也保留，例如布尔开关。
- 整行裁剪空格、tab 和 CR；赋值 key 只裁剪空格和 tab，因此 `title\r=value` 的 CR 属于键。NUL 也是普通行内数据，注释和裸 key 都不能在它处提前结束。外部 scanner 先确定赋值或裸项的上下文，再应用对应裁剪规则。
- 值节点保留去除外部空白后的完整原文。只有首尾均为双引号且长度至少为二时产生 `quoted_value`；内部双引号不会结束字符串，未闭合双引号与单引号属于原始值，不做通用转义解码。
- 不枚举已知配置键，也不全局推断 boolean/number/color。`title = true` 是标题字符串，`cursor-style-blink = true` 的值由该字段的解析器解释。这样新配置项不需要修改 grammar 即可高亮。

生成后的 crate 暴露 `LANGUAGE`、`GRAMMAR` 和 `NODE_TYPES`，聚合 crate 暴露对应的 `GHOSTTY_*`。kat 的 query、字段语义和 detector 属于 kat 主仓库，不能放进这里。

## 测试层次

普通 `just test` 检查 AST 的 key/value 字节范围、双引号分类、行边界、BOM、注释、空值以及结构样例，无需 Ghostty。本仓库的测试保存在独立 `ghostty-conformance` crate，生成 parser 不会覆盖测试。

需要官方 CLI 的两个测试默认 `ignore`，只由显式入口运行：

```bash
GHOSTTY_BIN=/path/to/ghostty just ghostty-check
```

1. 将合法配置样例与针对字符串边界生成的变体交给 grammar 和 `ghostty +validate-config --config-file=...`，任一失败都报错；负例还验证官方 CLI 确实返回诊断，避免错误的命令调用或空操作被误当成成功。
2. 获取 `ghostty +show-config --default --docs` 的管道输出，用 grammar 解析所有当前默认键和文档，并核对各行与 AST 的字段数量、值和范围。这能检测上游输出语法的漂移，不能代替非默认值样例。

默认输出不额外要求通过官方自校验：Ghostty 当前可能输出其自身拒绝重新载入的默认命令项。该问题不能通过删掉默认字段或忽略 grammar 错误来掩盖。

现有 Linux CI 继续执行普通检查。独立 [Ghostty Conformance workflow](../.github/workflows/ghostty-conformance.yml) 使用 macOS 的官方预编译 CLI：PR/push 校验官网当前稳定发布，每日及手动触发同时校验稳定版和 tip。下载地址、文件 SHA-256 与 `+version` 输出写入日志；下载或验证失败不会静默跳过。文件只解压到 runner 临时目录，不安装或启动终端 GUI。

## 更新与反例

维护时从官方解析代码及实际 CLI 行为出发。只有确认现有实现违反文件语法契约的反例才补回归测试，再修改源码并重新生成。Ghostty 对未知键、非法字号、不存在的文件等 schema 或环境问题的拒绝，不要求语法高亮 grammar 同样拒绝。任何关于拒绝行为的测试必须明确这是文件语法、字段校验还是外部资源问题。

固定样例无法自动覆盖所有上游新功能。动态默认输出与 tip 定期检查用于提前发现漂移；涉及非默认语法的更新仍需阅读上游代码并加入有依据的反例。
