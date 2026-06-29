use std::path::PathBuf;

use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer};

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::map::RecipeMap;
use crate::recipe::path::current_path;

/// # Files
/// Files, directories, or symlinks to place in the image.
///
/// Key is the destination path inside the image. For each entry, set exactly
/// one of `content`, `path`, or `symlink`.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
#[schemars(example = r#"files:
  /etc/motd:
    content: Welcome to Azari
    owner: root
    group: root
    chmod: 644
  /usr/local/bin/my-tool:
    path: ./assets/my-tool
    chmod: 755
  /etc/localtime:
    symlink: /usr/share/zoneinfo/UTC
"#)]
pub struct FilesField(RecipeMap<String, FileEntry>);

/// Describes a single file to be placed in the container image.
#[derive(Debug)]
pub struct FileEntry {
    pub source: FileSource,
    pub meta: FileMetadata,
}

/// The source of a file's content in a [`FilesField`] entry.
#[derive(Debug)]
pub enum FileSource {
    /// Inline file content written directly in the recipe.
    Content(String),
    /// Path to an existing file to be copied into the build directory.
    /// Resolved relative to the recipe file that defines it.
    Path(PathBuf),
    /// Symlink target path inside the image.
    Symlink(String),
}

/// Ownership and permission metadata for a [`FileEntry`].
#[derive(Debug, Default)]
pub struct FileMetadata {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub chmod: Option<i32>,
}

/// # File Entry
/// Path to the file.
#[derive(Deserialize, JsonSchema)]
#[allow(dead_code)]
#[serde(rename_all = "kebab-case")]
struct FileEntrySchema {
    /// # Content
    /// Inline content to write to the destination path.
    #[schemars(example = r#"/etc/motd:
  content: |
    Welcome to Azari,
    a declarative Linux system.
"#)]
    content: Option<String>,
    /// # Path
    /// Path to a local source file or directory.
    ///
    /// Relative paths are resolved from the config file location.
    path: Option<String>,
    /// # Symlink
    /// Symlink target path.
    symlink: Option<String>,
    /// # Owner
    /// File owner for copy operations.
    owner: Option<String>,
    /// # Group
    /// File group for copy operations.
    group: Option<String>,
    /// # Chmod
    /// File mode, e.g. `644` or `755`.
    chmod: Option<i32>,
}

impl schemars::JsonSchema for FileEntry {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FileEntry".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        FileEntrySchema::json_schema(generator)
    }
}

impl RecipeField for FilesField {
    type Value = Vec<(String, FileEntry)>;

    fn name() -> Option<&'static str> {
        Some("files")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |field| {
            format!("files:\"{}\"", field.unwrap_or_default())
        })
    }
}

impl Build for FilesField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        for (target, entry) in self.value()? {
            build_entry(builder, &target, entry)?;
        }

        Ok(())
    }
}

fn build_entry(builder: &mut Builder, target: &str, entry: FileEntry) -> Result<(), RecipeError> {
    let meta = entry.meta;
    match entry.source {
        FileSource::Content(content) => build_content_entry(builder, target, content, meta),
        FileSource::Path(src_path) => build_path_entry(builder, target, src_path, meta),
        FileSource::Symlink(symlink_target) => {
            build_symlink_entry(builder, target, &symlink_target, meta)
        }
    }
}

fn build_content_entry(
    builder: &mut Builder,
    target: &str,
    content: String,
    meta: FileMetadata,
) -> Result<(), RecipeError> {
    let filename = target_to_filename(target);
    let dest = builder.build_dir().join(&filename);

    std::fs::write(&dest, content)?;
    builder.push(copy_instruction(&filename, target, &meta));
    Ok(())
}

fn build_path_entry(
    builder: &mut Builder,
    target: &str,
    src_path: PathBuf,
    meta: FileMetadata,
) -> Result<(), RecipeError> {
    let filename = target_to_filename(target);
    let dest = builder.build_dir().join(&filename);

    copy_path_to_dest(&src_path, &dest)?;
    builder.push(copy_instruction(&filename, target, &meta));
    Ok(())
}

fn copy_path_to_dest(src: &std::path::Path, dest: &std::path::Path) -> Result<(), RecipeError> {
    let res = if src.is_dir() {
        std::fs::create_dir_all(dest)?;
        let opts = fs_extra::dir::CopyOptions {
            copy_inside: true,
            ..Default::default()
        };
        fs_extra::dir::copy(src, dest, &opts)
    } else {
        fs_extra::file::copy(src, dest, &Default::default())
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(std::io::Error::other(e).into()),
    }
}

fn build_symlink_entry(
    builder: &mut Builder,
    target: &str,
    symlink_target: &str,
    meta: FileMetadata,
) -> Result<(), RecipeError> {
    builder.push(format!(
        "RUN ln -sf {} {}",
        shell_quote(symlink_target),
        shell_quote(target)
    ));
    if let Some(chown) = chown_string(&meta) {
        builder.push(format!("RUN chown -h {chown} {}", shell_quote(target)));
    }
    if let Some(mode) = meta.chmod {
        builder.push(format!("RUN chmod {mode} {}", shell_quote(target)));
    }
    Ok(())
}

impl<'de> Deserialize<'de> for FileEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = FileEntrySchema::deserialize(deserializer)?;

        let source = match (raw.content, raw.path, raw.symlink) {
            (Some(content), None, None) => FileSource::Content(content),
            (None, Some(path), None) => {
                let config_file = current_path()
                    .ok_or_else(|| serde::de::Error::custom("current config path is not set"))?;
                let base = config_file
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."));
                FileSource::Path(base.join(path))
            }
            (None, None, Some(symlink)) => FileSource::Symlink(symlink),
            _ => {
                return Err(serde::de::Error::custom(
                    "file entry must have exactly one of: content, path, symlink",
                ));
            }
        };

        Ok(FileEntry {
            source,
            meta: FileMetadata {
                owner: raw.owner,
                group: raw.group,
                chmod: raw.chmod,
            },
        })
    }
}

/// Derives a build-directory filename from a container target path.
///
/// All characters that are not alphanumeric, `.`, or `-` are replaced with `_`.
///
/// `/etc/host name` → `etc_host_name`
pub(crate) fn target_to_filename(target: &str) -> String {
    let stripped = target.trim_start_matches('/');
    stripped
        .chars()
        .map(|c| match c.is_alphanumeric() || c == '.' || c == '-' {
            true => c,
            false => '_',
        })
        .collect()
}

/// Wraps `s` in single quotes for use as a shell argument.
///
/// Any single quotes within `s` are replaced with `'\''`.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn copy_instruction(src: &str, dst: &str, meta: &FileMetadata) -> String {
    let mut parts = vec!["COPY".to_owned()];

    if let Some(mode) = &meta.chmod {
        parts.push(format!("--chmod={mode}"));
    }

    if let Some(chown) = chown_string(meta) {
        parts.push(format!("--chown={chown}"));
    }

    if src.contains(' ') || dst.contains(' ') {
        // JSON array form handles paths that contain spaces.
        // Flags (--chmod, --chown) are placed before the array per BuildKit syntax.
        let src_j = src.replace('\\', "\\\\").replace('"', "\\\"");
        let dst_j = dst.replace('\\', "\\\\").replace('"', "\\\"");
        parts.push(format!("[\"{src_j}\", \"{dst_j}\"]"));
    } else {
        parts.push(src.to_owned());
        parts.push(dst.to_owned());
    }

    parts.join(" ")
}

fn chown_string(meta: &FileMetadata) -> Option<String> {
    let FileMetadata { owner, group, .. } = meta;
    match (owner, group) {
        (None, None) => None,
        (Some(o), Some(g)) => Some(format!("{o}:{g}")),
        (Some(o), None) => Some(o.to_owned()),
        (None, Some(g)) => Some(format!(":{g}")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use merge::Merge;

    use crate::recipe::error::RecipeError;
    use crate::recipe::field::RecipeField;
    use crate::recipe::path::SourcePathGuard;

    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn filename_simple() {
        // spaces, parens, colons all become underscores
        assert_eq!(target_to_filename("/etc/my-file"), "etc_my-file");
    }

    #[test]
    fn filename_replace_special_chars() {
        // spaces, parens, colons all become underscores
        assert_eq!(target_to_filename("/etc/a b:c(d)?!"), "etc_a_b_c_d___");
    }

    #[test]
    fn filename_unicode_support() {
        assert_eq!(target_to_filename("/etc/配置"), "etc_配置");
    }

    #[test]
    fn shell_quote_path() {
        assert_eq!(shell_quote("/etc/it's my file"), "'/etc/it'\\''s my file'");
    }

    #[test]
    fn copy_instruction_minimal() {
        assert_eq!(
            copy_instruction("etc_motd--abc123", "/etc/motd", &FileMetadata::default()),
            "COPY etc_motd--abc123 /etc/motd"
        );
    }

    #[test]
    fn copy_instruction_with_chmod_only() {
        let meta = FileMetadata {
            owner: None,
            group: None,
            chmod: Some(644),
        };
        assert_eq!(
            copy_instruction("src", "/dst", &meta),
            "COPY --chmod=644 src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_owner_only() {
        let meta = FileMetadata {
            owner: Some("root".to_string()),
            group: None,
            chmod: None,
        };
        assert_eq!(
            copy_instruction("src", "/dst", &meta),
            "COPY --chown=root src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_group_only() {
        let meta = FileMetadata {
            owner: None,
            group: Some("wheel".to_string()),
            chmod: None,
        };
        assert_eq!(
            copy_instruction("src", "/dst", &meta),
            "COPY --chown=:wheel src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_owner_and_group() {
        let meta = FileMetadata {
            owner: Some("root".to_string()),
            group: Some("wheel".to_string()),
            chmod: None,
        };
        assert_eq!(
            copy_instruction("src", "/dst", &meta),
            "COPY --chown=root:wheel src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_all_flags() {
        let meta = FileMetadata {
            owner: Some("user".to_string()),
            group: Some("grp".to_string()),
            chmod: Some(755),
        };
        assert_eq!(
            copy_instruction("src", "/dst", &meta),
            "COPY --chmod=755 --chown=user:grp src /dst"
        );
    }

    #[test]
    fn copy_instruction_dst_with_spaces_uses_quoted_form() {
        assert_eq!(
            copy_instruction("src", "/path with spaces", &FileMetadata::default()),
            r#"COPY ["src", "/path with spaces"]"#
        );
    }

    #[test]
    fn copy_instruction_dst_with_spaces_and_flags() {
        let meta = FileMetadata {
            owner: Some("root".to_string()),
            group: None,
            chmod: Some(644),
        };
        assert_eq!(
            copy_instruction("src", "/dst file", &meta),
            r#"COPY --chmod=644 --chown=root ["src", "/dst file"]"#
        );
    }

    #[test]
    fn default_is_empty() {
        let field = FilesField::default();
        assert_eq!(field.value().unwrap().len(), 0);
    }

    #[test]
    fn null_deserializes_to_default() {
        let _guard = SourcePathGuard::push_path(p("/recipe.yaml"));
        let field: FilesField = serde_saphyr::from_str("~").unwrap();
        assert_eq!(field.value().unwrap().len(), 0);
    }

    #[test]
    fn deserialize_content_entry() {
        let _guard = SourcePathGuard::push_path(p("/recipe.yaml"));
        let field: FilesField =
            serde_saphyr::from_str("/etc/motd:\n  content: hello world\n").unwrap();
        let mut entries = field.value().unwrap();
        assert_eq!(entries.len(), 1);
        let (target, entry) = entries.remove(0);
        assert_eq!(target, "/etc/motd");
        assert!(matches!(&entry.source, FileSource::Content(s) if s == "hello world"));
        assert!(entry.meta.owner.is_none());
        assert!(entry.meta.group.is_none());
        assert!(entry.meta.chmod.is_none());
    }

    #[test]
    fn deserialize_content_entry_with_metadata() {
        let _guard = SourcePathGuard::push_path(p("/recipe.yaml"));
        let field: FilesField = serde_saphyr::from_str(
            "/etc/motd:\n  content: hello\n  owner: root\n  group: wheel\n  chmod: '644'\n",
        )
        .unwrap();
        let mut entries = field.value().unwrap();
        let (_, entry) = entries.remove(0);
        assert_eq!(entry.meta.owner.as_deref(), Some("root"));
        assert_eq!(entry.meta.group.as_deref(), Some("wheel"));
        assert_eq!(entry.meta.chmod, Some(644));
    }

    #[test]
    fn deserialize_symlink_entry() {
        let _guard = SourcePathGuard::push_path(p("/recipe.yaml"));
        let field: FilesField =
            serde_saphyr::from_str("/usr/bin/sh:\n  symlink: /usr/bin/bash\n").unwrap();
        let mut entries = field.value().unwrap();
        let (target, entry) = entries.remove(0);
        assert_eq!(target, "/usr/bin/sh");
        assert!(matches!(&entry.source, FileSource::Symlink(s) if s == "/usr/bin/bash"));
    }

    #[test]
    fn deserialize_path_entry_resolves_relative_to_recipe() {
        let _guard = SourcePathGuard::push_path(p("/some/dir/recipe.yaml"));
        let field: FilesField =
            serde_saphyr::from_str("/etc/config:\n  path: files/config.conf\n").unwrap();
        let mut entries = field.value().unwrap();
        let (_, entry) = entries.remove(0);
        assert!(
            matches!(&entry.source, FileSource::Path(p) if p == &PathBuf::from("/some/dir/files/config.conf"))
        );
    }

    #[test]
    fn deserialize_multiple_sources_is_error() {
        let _guard = SourcePathGuard::push_path(p("/recipe.yaml"));
        let result: Result<FilesField, _> =
            serde_saphyr::from_str("/etc/motd:\n  content: hello\n  symlink: /other\n");
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_missing_source_is_error() {
        let _guard = SourcePathGuard::push_path(p("/recipe.yaml"));
        let result: Result<FilesField, _> = serde_saphyr::from_str("/etc/motd:\n  owner: root\n");
        assert!(result.is_err());
    }

    #[test]
    fn merge_combines_entries_from_both() {
        let _guard1 = SourcePathGuard::push_path(p("/a.yaml"));
        let mut merged: FilesField = serde_saphyr::from_str("/etc/a:\n  content: aaa\n").unwrap();
        let _guard2 = SourcePathGuard::push_path(p("/b.yaml"));
        let b: FilesField = serde_saphyr::from_str("/etc/b:\n  content: bbb\n").unwrap();
        merged.merge(b);
        let entries = merged.value().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "/etc/a");
        assert_eq!(entries[1].0, "/etc/b");
    }

    #[test]
    fn duplicate_target_across_sources_is_conflict() {
        let _guard1 = SourcePathGuard::push_path(p("/a.yaml"));
        let mut merged: FilesField =
            serde_saphyr::from_str("/etc/same:\n  content: aaa\n").unwrap();
        let _guard2 = SourcePathGuard::push_path(p("/b.yaml"));
        let b: FilesField = serde_saphyr::from_str("/etc/same:\n  content: bbb\n").unwrap();
        merged.merge(b);
        assert!(matches!(
            merged.value(),
            Err(RecipeError::FieldConflict { .. })
        ));
    }
}
