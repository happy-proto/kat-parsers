//! 根据显式来源清单批量检查、同步和重放上游快照。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct Metadata {
    language: String,
    sync: Source,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    #[serde(default)]
    local: bool,
    repository: Option<String>,
    revision: Option<String>,
    #[serde(default = "head")]
    reference: String,
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    patches: Vec<String>,
}

fn head() -> String {
    "HEAD".into()
}

struct Prepared {
    metadata_path: PathBuf,
    metadata_text: String,
    language: String,
    stage: PathBuf,
    files: Vec<PathBuf>,
    removed: Vec<PathBuf>,
    revision: String,
}

pub fn sync(root: &Path, language: Option<&str>, check: bool, locked: bool) -> Result<()> {
    let mut paths = fs::read_dir(root.join("metadata"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "toml")
    });
    paths.sort();
    let temporary = TempDir::new()?;
    let mut repositories = BTreeMap::<(String, String), (PathBuf, String)>::new();
    let mut prepared = Vec::new();
    let mut changes = Vec::new();
    let mut matched = false;

    for path in paths {
        let text = fs::read_to_string(&path)?;
        let value: toml::Value = toml::from_str(&text)?;
        let name = value
            .get("language")
            .and_then(toml::Value::as_str)
            .context("metadata 缺少 language")?;
        if language.is_some_and(|wanted| name != wanted) {
            continue;
        }
        matched = true;
        let metadata: Metadata = toml::from_str(&text)
            .with_context(|| format!("{} 缺少有效的 [sync] 来源约定", path.display()))?;
        validate(&metadata)?;
        if metadata.sync.local {
            println!("{}: 本仓库维护，跳过远程同步", metadata.language);
            continue;
        }
        let source = &metadata.sync;
        let repository = source.repository.as_deref().unwrap();
        let previous = source.revision.as_deref().unwrap();
        // 检查模式只查询引用，不下载源码；锁定检查则重放并比较实际资产。
        if check && !locked {
            let revision = remote_revision(repository, &source.reference)?;
            if revision != previous {
                changes.push(format!("{}: {previous} -> {revision}", metadata.language));
            }
            continue;
        }
        let reference = if locked { previous } else { &source.reference };
        let key = (repository.to_owned(), reference.to_owned());
        let repository_index = repositories.len();
        if !repositories.contains_key(&key) {
            let checkout = temporary
                .path()
                .join(format!("repository-{repository_index}"));
            fs::create_dir(&checkout)?;
            git(&checkout, &["init", "--quiet"])?;
            git(
                &checkout,
                &["fetch", "--quiet", "--depth=1", repository, reference],
            )?;
            git(
                &checkout,
                &["checkout", "--quiet", "--detach", "FETCH_HEAD"],
            )?;
            let revision = git(&checkout, &["rev-parse", "HEAD"])?;
            repositories.insert(key.clone(), (checkout, revision));
        }
        let (checkout, revision) = &repositories[&key];
        let stage = temporary
            .path()
            .join(format!("stage-{}", metadata.language));
        fs::create_dir(&stage)?;
        for (destination, origin) in &source.files {
            let origin_path = checkout.join(origin);
            if !origin_path
                .canonicalize()?
                .starts_with(checkout.canonicalize()?)
            {
                bail!("{}: 来源资产越过仓库边界: {origin}", metadata.language);
            }
            copy_asset(&origin_path, &stage.join(destination))
                .with_context(|| format!("{}: {origin} -> {destination}", metadata.language))?;
        }
        for patch in &source.patches {
            let patch_path = root
                .join(patch)
                .canonicalize()
                .with_context(|| format!("缺少补丁 {patch}"))?;
            git(&stage, &["init", "--quiet"])?;
            git(
                &stage,
                &[
                    "apply",
                    "--check",
                    patch_path.to_str().context("非 UTF-8 补丁路径")?,
                ],
            )?;
            git(
                &stage,
                &["apply", patch_path.to_str().context("非 UTF-8 补丁路径")?],
            )?;
        }
        let files = asset_files(&stage)?;
        if files.is_empty() {
            bail!("{}: 来源映射没有产生资产", metadata.language);
        }
        let current = root.join("upstreams").join(&metadata.language);
        reject_symlinks(root, &current)?;
        let existing = if current.exists() {
            asset_files(&current)?
        } else {
            Vec::new()
        };
        let removed = existing
            .into_iter()
            .filter(|file| !files.contains(file))
            .collect::<Vec<_>>();
        let content_changed = !removed.is_empty()
            || files
                .iter()
                .any(|file| fs::read(current.join(file)).ok() != fs::read(stage.join(file)).ok());
        let assets = files
            .iter()
            .map(|file| file.to_string_lossy().into_owned())
            .collect::<BTreeSet<_>>();
        if content_changed
            || revision != previous
            || declared_assets(&value).as_ref() != Some(&assets)
        {
            changes.push(format!("{}: {previous} -> {revision}", metadata.language));
        }
        prepared.push(Prepared {
            metadata_path: path,
            metadata_text: text,
            language: metadata.language,
            stage,
            files,
            removed,
            revision: revision.clone(),
        });
    }
    if !matched {
        bail!("未知 grammar: {}", language.unwrap_or("<all>"));
    }
    if check {
        if changes.is_empty() {
            println!("所有上游来源均已核对，未发现差异");
            return Ok(());
        }
        for change in &changes {
            println!("{change}");
        }
        bail!("{} 个上游来源需要同步", changes.len());
    }
    // 先准备所有写入与回滚数据；路径冲突也必须在写入第一种语言之前暴露。
    let mut mutations = Vec::new();
    for item in &prepared {
        for file in &item.files {
            let destination = root.join("upstreams").join(&item.language).join(file);
            let bytes = fs::read(item.stage.join(file))?;
            Mutation::plan(root, destination, Some(bytes), &mut mutations)?;
        }
        for file in &item.removed {
            Mutation::plan(
                root,
                root.join("upstreams").join(&item.language).join(file),
                None,
                &mut mutations,
            )?;
        }
        let mut value: toml::Value = toml::from_str(&item.metadata_text)?;
        let old_revision = value["sync"]["revision"].as_str().unwrap();
        let declared = declared_assets(&value);
        let assets = item
            .files
            .iter()
            .map(|file| file.to_string_lossy().into_owned())
            .collect::<BTreeSet<_>>();
        if old_revision != item.revision || declared.as_ref() != Some(&assets) {
            value["sync"]["revision"] = item.revision.clone().into();
            for key in ["upstream_commit", "upstream_revision"] {
                if value.get(key).is_some() {
                    value[key] = item.revision.clone().into();
                }
            }
            if value.get("copied").is_none() {
                value
                    .as_table_mut()
                    .context("metadata 必须是 TOML table")?
                    .insert("copied".into(), toml::Value::Table(Default::default()));
            }
            value
                .get_mut("copied")
                .context("缺少 copied")?
                .as_table_mut()
                .context("copied 必须是 TOML table")?
                .insert(
                    "files".into(),
                    toml::Value::Array(assets.into_iter().map(toml::Value::String).collect()),
                );
            Mutation::plan(
                root,
                item.metadata_path.clone(),
                Some(toml::to_string_pretty(&value)?.into_bytes()),
                &mut mutations,
            )?;
        }
    }
    for (index, mutation) in mutations.iter().enumerate() {
        if let Err(error) = mutation.write(mutation.next.as_deref()) {
            let rollback = mutations[..index]
                .iter()
                .rev()
                .try_for_each(|previous| previous.write(previous.previous.as_deref()));
            if let Err(rollback_error) = rollback {
                bail!("同步写入失败: {error:#}; 回滚失败，请检查工作区: {rollback_error:#}");
            }
            return Err(error.context("同步写入失败，已回滚先前资产"));
        }
    }
    for item in prepared {
        println!("{}: {}", item.language, item.revision);
    }
    println!("同步完成；运行 generate 重新生成 parser，并执行仓库检查");
    Ok(())
}

fn declared_assets(value: &toml::Value) -> Option<BTreeSet<String>> {
    value
        .get("copied")
        .and_then(|value| value.get("files"))
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
}

struct Mutation {
    path: PathBuf,
    previous: Option<Vec<u8>>,
    next: Option<Vec<u8>>,
}

impl Mutation {
    fn plan(
        root: &Path,
        path: PathBuf,
        next: Option<Vec<u8>>,
        mutations: &mut Vec<Self>,
    ) -> Result<()> {
        reject_symlinks(root, &path)?;
        let previous = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(error).with_context(|| format!("无法准备 {}", path.display()));
            }
        };
        if previous != next {
            mutations.push(Self {
                path,
                previous,
                next,
            });
        }
        Ok(())
    }

    fn write(&self, bytes: Option<&[u8]>) -> Result<()> {
        if let Some(bytes) = bytes {
            let parent = self.path.parent().unwrap();
            fs::create_dir_all(parent)?;
            let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
            use std::io::Write;
            temporary.write_all(bytes)?;
            if let Ok(metadata) = fs::metadata(&self.path) {
                temporary
                    .as_file()
                    .set_permissions(metadata.permissions())?;
            }
            temporary.persist(&self.path).map_err(|error| error.error)?;
        } else if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

fn reject_symlinks(root: &Path, target: &Path) -> Result<()> {
    let relative = target.strip_prefix(root)?;
    let mut path = root.to_path_buf();
    for part in relative.components() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!("同步路径不能包含软链接: {}", path.display())
            }
            Ok(metadata) if path != target && !metadata.is_dir() => {
                bail!("同步父路径不是目录: {}", path.display())
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn validate(metadata: &Metadata) -> Result<()> {
    safe_path(&metadata.language)?;
    if Path::new(&metadata.language).components().count() != 1 {
        bail!("language 必须是单个目录名");
    }
    let source = &metadata.sync;
    if source.local {
        if source.repository.is_some()
            || source.revision.is_some()
            || !source.files.is_empty()
            || !source.patches.is_empty()
        {
            bail!("{}: local 来源不能混用远程同步设置", metadata.language);
        }
        return Ok(());
    }
    if source.repository.as_deref().is_none_or(str::is_empty) {
        bail!("{}: 缺少 sync.repository", metadata.language);
    }
    let revision = source.revision.as_deref().context("缺少 sync.revision")?;
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{}: sync.revision 必须是完整 Git SHA", metadata.language);
    }
    if source.reference.is_empty() || source.reference.starts_with('-') || source.files.is_empty() {
        bail!(
            "{}: 无效的 sync.reference 或空的文件映射",
            metadata.language
        );
    }
    for (destination, origin) in &source.files {
        safe_path(destination)?;
        safe_path(origin)?;
    }
    for patch in &source.patches {
        safe_path(patch)?;
    }
    Ok(())
}

fn safe_path(path: &str) -> Result<()> {
    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("来源路径必须是非空、安全的相对路径: {path}");
    }
    Ok(())
}

fn remote_revision(repository: &str, reference: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["ls-remote", repository, reference])
        .output()
        .context("无法查询上游 Git 引用")?;
    if !output.status.success() {
        bail!(
            "查询 {repository} 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8(output.stdout)?;
    let mut lines = stdout.lines();
    let revision = lines
        .next()
        .and_then(|line| line.split_whitespace().next())
        .context("上游引用不存在")?;
    if lines.next().is_some() {
        bail!("上游引用不唯一，请使用完整 ref: {reference}");
    }
    Ok(revision.to_owned())
}

fn git(directory: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()?;
    if !output.status.success() {
        bail!("Git 操作失败: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn copy_asset(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        bail!("同步资产不能是软链接: {}", source.display());
    }
    if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_asset(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if metadata.is_file() {
        fs::create_dir_all(destination.parent().unwrap())?;
        fs::copy(source, destination)?;
    } else {
        bail!("不支持的资产类型: {}", source.display());
    }
    Ok(())
}

fn asset_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_name() == ".git" {
                continue;
            }
            if entry.file_type()?.is_symlink() {
                bail!("同步资产不能包含软链接: {}", entry.path().display());
            }
            if entry.file_type()?.is_dir() {
                stack.push(entry.path());
            } else {
                files.push(entry.path().strip_prefix(root)?.to_path_buf());
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<(TempDir, TempDir, String)> {
        let repository = TempDir::new()?;
        fs::write(repository.path().join("grammar.js"), "original")?;
        git(repository.path(), &["init", "--quiet"])?;
        git(repository.path(), &["add", "grammar.js"])?;
        git(
            repository.path(),
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "initial",
            ],
        )?;
        let revision = git(repository.path(), &["rev-parse", "HEAD"])?;
        let root = TempDir::new()?;
        fs::create_dir(root.path().join("metadata"))?;
        fs::write(
            root.path().join("metadata/example.toml"),
            format!(
                "language = \"example\"\n[sync]\nrepository = {:?}\nrevision = {:?}\n[sync.files]\n\"grammar.js\" = \"grammar.js\"\n",
                repository.path().to_str().unwrap(),
                revision
            ),
        )?;
        Ok((repository, root, revision))
    }

    #[test]
    fn pinned_replay_is_idempotent_and_detects_local_drift() -> Result<()> {
        let (_repository, root, _) = fixture()?;
        sync(root.path(), None, false, true)?;
        let grammar = root.path().join("upstreams/example/grammar.js");
        let modified = fs::metadata(&grammar)?.modified()?;
        sync(root.path(), None, true, true)?;
        sync(root.path(), None, false, true)?;
        assert_eq!(fs::metadata(&grammar)?.modified()?, modified);
        fs::write(grammar, "local drift")?;
        assert!(sync(root.path(), None, true, true).is_err());
        Ok(())
    }

    #[test]
    fn batch_update_checks_remote_and_advances_pinned_revision() -> Result<()> {
        let (repository, root, previous) = fixture()?;
        fs::write(repository.path().join("grammar.js"), "updated")?;
        git(repository.path(), &["add", "grammar.js"])?;
        git(
            repository.path(),
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "update",
            ],
        )?;
        assert!(sync(root.path(), None, true, false).is_err());
        sync(root.path(), None, false, false)?;
        let value: toml::Value = toml::from_str(&fs::read_to_string(
            root.path().join("metadata/example.toml"),
        )?)?;
        assert_ne!(value["sync"]["revision"].as_str(), Some(previous.as_str()));
        assert_eq!(
            fs::read_to_string(root.path().join("upstreams/example/grammar.js"))?,
            "updated"
        );
        sync(root.path(), None, true, false)?;
        Ok(())
    }

    #[test]
    fn missing_source_fails_before_writing_any_language() -> Result<()> {
        let (_repository, root, _) = fixture()?;
        let text = fs::read_to_string(root.path().join("metadata/example.toml"))?;
        fs::write(
            root.path().join("metadata/second.toml"),
            text.replace("\"example\"", "\"second\"")
                .replace("= \"grammar.js\"", "= \"missing.js\""),
        )?;
        assert!(sync(root.path(), None, false, true).is_err());
        assert!(!root.path().join("upstreams").exists());
        Ok(())
    }

    #[test]
    fn unknown_language_and_parent_paths_are_rejected() -> Result<()> {
        let (_repository, root, _) = fixture()?;
        assert!(sync(root.path(), Some("unknown"), false, true).is_err());
        for path in ["", "..", "../outside", "/absolute", "a/../../outside"] {
            assert!(safe_path(path).is_err());
        }
        Ok(())
    }

    #[test]
    fn replay_removes_obsolete_assets_and_detects_them_in_check_mode() -> Result<()> {
        let (_repository, root, _) = fixture()?;
        sync(root.path(), None, false, true)?;
        let obsolete = root.path().join("upstreams/example/obsolete.js");
        fs::write(&obsolete, "no longer imported")?;
        assert!(sync(root.path(), None, true, true).is_err());
        sync(root.path(), None, false, true)?;
        assert!(!obsolete.exists());
        sync(root.path(), None, true, true)?;
        Ok(())
    }

    #[test]
    fn later_destination_conflict_preserves_the_entire_batch() -> Result<()> {
        let (_repository, root, _) = fixture()?;
        sync(root.path(), None, false, true)?;
        let grammar = root.path().join("upstreams/example/grammar.js");
        fs::write(&grammar, "keep local changes on failed batch")?;
        let metadata = fs::read_to_string(root.path().join("metadata/example.toml"))?;
        fs::write(
            root.path().join("metadata/second.toml"),
            metadata.replace("\"example\"", "\"second\""),
        )?;
        fs::create_dir_all(root.path().join("upstreams/second/grammar.js"))?;
        assert!(sync(root.path(), None, false, true).is_err());
        assert_eq!(
            fs::read_to_string(grammar)?,
            "keep local changes on failed batch"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn destination_symlink_is_rejected_without_changing_external_file() -> Result<()> {
        let (_repository, root, _) = fixture()?;
        let outside = TempDir::new()?;
        let sentinel = outside.path().join("sentinel");
        fs::write(&sentinel, "untouched")?;
        fs::create_dir_all(root.path().join("upstreams/example"))?;
        std::os::unix::fs::symlink(&sentinel, root.path().join("upstreams/example/grammar.js"))?;
        assert!(sync(root.path(), None, false, true).is_err());
        assert_eq!(fs::read_to_string(sentinel)?, "untouched");
        Ok(())
    }
}
