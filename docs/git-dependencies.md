# 同步 Git 依赖

Kat 和 Sidediff 通过 `Cargo.lock` 固定直接与间接 Git 依赖的 revision。Cargo 不会读取依赖仓库自己的 lockfile，因此上游 Git crate 的 lockfile 更新不会自动传播到应用仓库；当前 [Dependabot Cargo 实现](https://github.com/dependabot/dependabot-core/blob/main/cargo/lib/dependabot/cargo/update_checker.rb) 也不支持更新间接 Git 依赖。

本仓库提供可复用的 [sync-git-dependencies.yml](../.github/workflows/sync-git-dependencies.yml)。它读取调用仓库的 `Cargo.lock` 依赖图，用 `cargo metadata --locked` 收集所有来源为 Git 的 package ID，再在一次 `cargo update <package-id>...` 中选择它们。命令直接传入 metadata 提供的完整 package ID，包含来源 URL、名称和版本，可区分同名依赖。间接 Git 依赖会从 Cargo metadata 结果中一并选择，调用方不需要在 `Cargo.toml` 直接声明它们。命令不使用 `--recursive`，不会主动升级无关的 registry 依赖。

调用仓库可添加定时和手动触发的轻量 workflow：

```yaml
name: Sync Git dependencies

on:
  schedule:
    - cron: "0 3 * * 1"
  workflow_dispatch:

jobs:
  sync:
    permissions:
      contents: write
      actions: write
    uses: happy-proto/kat-parsers/.github/workflows/sync-git-dependencies.yml@FULL_COMMIT_SHA
    with:
      ci_workflow: ci.yml
```

将 `FULL_COMMIT_SHA` 替换为已审查的完整提交 SHA，并通过调用方的 Actions 依赖更新流程维护。调用仓库需要允许 workflow token 写入内容并调度 Actions。`ci_workflow` 是唯一输入，默认值为 `ci.yml`；该 workflow 必须支持 `workflow_dispatch`。

已有更新分支没有新改动时仍会重新调度 CI，以便恢复上次推送成功、调度中断的情况。同步 job 成功只表示调度请求被接受，实际 CI 结果需独立检查。

可复用 workflow 从默认分支开始工作，并复用 `chore/sync-git-dependencies` 分支：已有分支会先合并默认分支。仅当 lockfile 或分支提交有变化时才推送，因此仅合并默认分支也会推送并触发 CI。它不会 force-push，也不会创建或合并 PR。组织禁止 Actions 创建 PR，所以 workflow 会在 summary 中提供带中文标题的 Compare 链接，由维护者审阅后创建 PR。成功推送后会显式调度调用方指定的 CI workflow；GitHub token 推送本身不会触发常规 CI。
