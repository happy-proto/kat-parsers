use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};

/// CLI 0.27.1 的 array_delete 宏会重复求值，破坏 XML 的 &array_pop(...)。
/// 用字节访问保留所有 Array(T) 布局，同时避免把不同 Array 类型互转的别名问题。
pub fn normalize_array_header(path: &Path) -> Result<()> {
    let mut source = fs::read_to_string(path)?;
    let start = source
        .find("#define array_delete(self)")
        .context("missing public array_delete macro")?;
    let end = start
        + source[start..]
            .find("\n\n")
            .context("unterminated public array_delete macro")?;
    ensure!(
        source[start..end].contains("(self)->contents")
            && source[start..end].contains("(self)->capacity"),
        "public array_delete changed; review scanner header compatibility before updating CLI"
    );
    source.replace_range(
        start..end,
        r#"static inline void _kat_array_delete(void *self, size_t size) {
  void *contents;
  memcpy(&contents, self, sizeof(contents));
  if (contents) ts_free(contents);
  memset(self, 0, size);
}
#define array_delete(self) _kat_array_delete((self), sizeof(*(self)))"#,
    );
    fs::write(path, source)?;
    Ok(())
}
