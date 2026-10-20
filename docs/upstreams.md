# 批量维护上游语法

所有语言的来源都由 `metadata/<language>.toml` 声明。应用只依赖预生成 parser crate；检查、同步、Tree-sitter 生成与 Zstd 压缩都在本仓库完成。

## 常用命令

```sh
# 一次检查所有远程来源，发现更新时返回非零
just check-upstreams

# 一次同步全部来源、重新生成并运行仓库检查
just sync

# 按提交中锁定的来源重放，验证资产和本地补丁没有漂移
just check-upstream-sources
```

也可直接使用 `cargo run --locked -p xtask -- sync <language>` 只更新某种语言。省略语言名处理全部语言；`--check` 不写工作区，`--locked` 重放记录的精确 Git revision，不追踪新提交。来源检查不要求 Node.js 或 Tree-sitter CLI，重新生成时才需要它们。

同步会先下载并准备整批来源、重放补丁并检查目标路径，再写入工作区。单个文件使用原子替换；后续写入失败时回滚先前写入。完整快照的文件新增与删除会一起同步，不通过软链接写入外部文件。内容相同的文件保留修改时间。建议从干净工作区运行；同步有意恢复声明的上游快照，本地改造应先转为补丁。

## 来源清单

远程来源示例：

```toml
[sync]
repository = "https://github.com/example/tree-sitter-example"
revision = "0123456789abcdef0123456789abcdef01234567"
reference = "HEAD"
patches = ["patches/example/paths.patch"]

[sync.files]
"grammar.js" = "grammar.js"
"scanner.c" = "src/scanner.c"
"LICENSE" = "LICENSE"
```

- `revision` 是当前快照的完整 Git SHA；`reference` 是后续检查的上游引用，默认跟踪上游默认分支。更新只修改锁定 SHA，不使用浮动内容作为构建输入。
- 文件映射左侧是 `upstreams/<language>/` 内的路径，右侧是上游仓库根目录内的路径。允许映射一个资产目录，完整同步其子文件；不允许绝对路径、父目录跳转或软链接。
- `patches` 是本仓库内的补丁路径。补丁以单语言快照根目录为基准，在安装快照前应用。路径适配、scanner 修复和本地 grammar 改造都需显式记录；补丁冲突必须修正，不能静默跳过。
- `copied.files` 记录生成器消费的完整资产集。目录映射产生的文件集变化会同步更新该字段；源码、许可证、第三方声明与来源元数据都保留。
- `upstream_version` 若存在，是最初迁移对应的 crate 版本；后续快照身份以 `sync.revision` 为准，不能用版本标签推断源码未变化。

自维护语言明确声明：

```toml
[sync]
local = true
```

这类语言没有远程源码可覆盖，由仓库回归维护。原仓库迁移或锁定提交无法获取时，使用保留原提交的公开镜像，或经过源码逐字节对照的官方提交。

## 第三方源码与最小补丁

第三方源码尽量按锁定上游的原始内容保存，不对它们运行格式化工具，也不为调整注释、增加来源声明或消除非致命告警建立补丁。来源、revision、许可证和本地改动原因统一记录在 metadata 与 `THIRD_PARTY.md`。

只保留生成兼容、资产路径适配、scanner 安全或解析行为所必需的最小差异。补丁名称应说明目的，不使用笼统的 `snapshot.patch` 来保存整份历史差异；修复和格式整理不要混在同一补丁中。上游已有模块化 grammar 时，直接同步所需模块，避免把依赖内联成另一份 grammar。

精简已有补丁时，先核对 grammar 与 node-types 的生成结果，保留现有节点和规则契约，再重新生成并验证原生 parser、共享压缩产物及消费方。`sync --locked --check` 会逐字节重放来源和剩余补丁，防止未记录的源码漂移。

某些上游 `grammar.js` 依赖特定版本的其它 JavaScript grammar。C++、Groovy、TypeScript 和 TSX 使用同一锁定版本的展开 `grammar.json` 作为生成入口，避免维护多套隐含 npm 依赖；仍由官方 CLI 生成 ABI 15 parser。scanner 和许可证照常独立同步。

## 定时同步

[Sync upstream grammars](../.github/workflows/sync-upstreams.yml) 每周运行，也可手动触发。它统一同步、生成并检查所有语言，将成功更新推送到 `chore/sync-upstreams`，并显式调度两项必需 CI。已有更新分支会合入默认分支，不强制推送或覆盖协作者提交。

当前组织策略禁止 GitHub Actions 创建 PR，因此 workflow summary 提供已填入中文标题和模板的比较链接。检查和源码同步不需要逐语言人工操作；只需通过链接提交一次更新 PR。流程不会自动合并，也不会修改组织策略。上游文件布局变化、补丁冲突和解析回归会使任务失败，需先适配后再重跑。
