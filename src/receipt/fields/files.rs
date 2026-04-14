use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::map::ReceiptMap;
use crate::receipt::path::current_path;

/// The source of a file's content in a [`FilesField`] entry.
#[derive(Debug)]
pub enum FileSource {
    /// Inline file content written directly in the receipt.
    Content(String),
    /// Path to an existing file to be copied into the build directory.
    /// Resolved relative to the receipt file that defines it.
    Path(PathBuf),
    /// Symlink target path inside the image.
    Symlink(String),
}

/// Describes a single file to be placed in the container image.
#[derive(Debug)]
pub struct FileEntry {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub chmod: Option<String>,
    pub source: FileSource,
}

impl<'de> Deserialize<'de> for FileEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Raw {
            owner: Option<String>,
            group: Option<String>,
            chmod: Option<String>,
            content: Option<String>,
            path: Option<PathBuf>,
            symlink: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let source = match (raw.content, raw.path, raw.symlink) {
            (Some(content), None, None) => FileSource::Content(content),
            (None, Some(path), None) => {
                let source_file = current_path().expect("Current source path is not set");
                let base = source_file
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
            owner: raw.owner,
            group: raw.group,
            chmod: raw.chmod,
            source,
        })
    }
}

/// Field for the `files` key.
///
/// A map from target paths (inside the image) to file descriptors. Each
/// descriptor specifies the file's source (`content`, `path`, or `symlink`)
/// and optional `owner`, `group`, and `chmod` attributes.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct FilesField(pub(crate) ReceiptMap<String, FileEntry>);

impl ReceiptField for FilesField {
    type Value = Vec<(String, FileEntry)>;

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn sources(&self) -> &[PathBuf] {
        self.0.sources()
    }

    fn merge(self, other: Self) -> Self {
        Self(self.0.merge(other.0))
    }
}

impl Build for FilesField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        let files = self.value()?;
        let build_dir = builder.build_dir().to_owned();

        for (target, entry) in files {
            match entry.source {
                FileSource::Content(content) => {
                    let filename = target_to_filename(&target, content.as_bytes());
                    let dest = build_dir.join(&filename);
                    std::fs::write(&dest, content)?;
                    builder.push(copy_instruction(
                        &filename,
                        &target,
                        &entry.owner,
                        &entry.group,
                        &entry.chmod,
                    ));
                }
                FileSource::Path(src_path) => {
                    let bytes = std::fs::read(&src_path)?;
                    let filename = target_to_filename(&target, &bytes);
                    let dest = build_dir.join(&filename);
                    std::fs::write(&dest, &bytes)?;
                    builder.push(copy_instruction(
                        &filename,
                        &target,
                        &entry.owner,
                        &entry.group,
                        &entry.chmod,
                    ));
                }
                FileSource::Symlink(symlink_target) => {
                    builder.push(format!(
                        "RUN ln -sf {} {}",
                        shell_quote(&symlink_target),
                        shell_quote(&target),
                    ));
                    if entry.owner.is_some() || entry.group.is_some() {
                        let chown = format_chown(&entry.owner, &entry.group);
                        builder.push(format!("RUN chown -h {chown} {}", shell_quote(&target)));
                    }
                    if let Some(mode) = &entry.chmod {
                        builder.push(format!("RUN chmod {mode} {}", shell_quote(&target)));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Derives a unique build-directory filename from a container target path.
///
/// All characters that are not alphanumeric, `.`, or `-` are replaced with `_`,
/// followed by `--` and the first 12 hex characters of the SHA-256 digest of `content`.
///
/// `/etc/host name` + content → `etc_host_name--<hash12>`
fn target_to_filename(target: &str, content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let stripped = target.trim_start_matches('/');
    let sanitized: String = stripped
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let hash = format!("{:x}", Sha256::digest(content));
    format!("{}--{}", sanitized, &hash[..12])
}

/// Wraps `s` in single quotes for use as a shell argument.
///
/// Any single quotes within `s` are replaced with `'\''`.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn copy_instruction(
    src: &str,
    dst: &str,
    owner: &Option<String>,
    group: &Option<String>,
    chmod: &Option<String>,
) -> String {
    let mut parts = vec!["COPY".to_owned()];

    if let Some(mode) = chmod {
        parts.push(format!("--chmod={mode}"));
    }

    if let Some(chown) = format_chown_opt(owner, group) {
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

fn format_chown(owner: &Option<String>, group: &Option<String>) -> String {
    match (owner, group) {
        (Some(o), Some(g)) => format!("{o}:{g}"),
        (Some(o), None) => o.clone(),
        (None, Some(g)) => format!(":{g}"),
        (None, None) => String::new(),
    }
}

fn format_chown_opt(owner: &Option<String>, group: &Option<String>) -> Option<String> {
    match (owner, group) {
        (None, None) => None,
        _ => Some(format_chown(owner, group)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::receipt::error::ReceiptError;
    use crate::receipt::field::ReceiptField;
    use crate::receipt::path::SourcePathGuard;

    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    // SHA256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    const EMPTY_HASH: &str = "e3b0c44298fc";

    // --- target_to_filename ---

    #[test]
    fn filename_strips_leading_slash() {
        assert_eq!(
            target_to_filename("/etc/hostname", b""),
            format!("etc_hostname--{EMPTY_HASH}")
        );
    }

    #[test]
    fn filename_without_leading_slash() {
        assert_eq!(
            target_to_filename("etc/hostname", b""),
            format!("etc_hostname--{EMPTY_HASH}")
        );
    }

    #[test]
    fn filename_deeply_nested_path() {
        assert_eq!(
            target_to_filename("/a/b/c/d", b""),
            format!("a_b_c_d--{EMPTY_HASH}")
        );
    }

    #[test]
    fn filename_spaces_are_replaced() {
        assert_eq!(
            target_to_filename("/etc/my file", b""),
            format!("etc_my_file--{EMPTY_HASH}")
        );
    }

    #[test]
    fn filename_special_chars_are_replaced() {
        // spaces, parens, colons all become underscores
        let name = target_to_filename("/etc/a b:c(d)", b"");
        assert!(
            !name.contains(' '),
            "name should not contain spaces: {name}"
        );
        assert!(name.starts_with("etc_a_b_c_d_--"));
    }

    #[test]
    fn filename_different_content_gives_different_hash() {
        let a = target_to_filename("/etc/f", b"hello");
        let b = target_to_filename("/etc/f", b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn filename_hash_suffix_is_12_lowercase_hex_chars() {
        let name = target_to_filename("/etc/test", b"some content");
        let suffix = name.split("--").nth(1).unwrap();
        assert_eq!(suffix.len(), 12);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // --- shell_quote ---

    #[test]
    fn shell_quote_plain_path() {
        assert_eq!(shell_quote("/etc/hostname"), "'/etc/hostname'");
    }

    #[test]
    fn shell_quote_path_with_spaces() {
        assert_eq!(shell_quote("/etc/my file"), "'/etc/my file'");
    }

    #[test]
    fn shell_quote_embedded_single_quote() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    // --- copy_instruction ---

    #[test]
    fn copy_instruction_minimal() {
        assert_eq!(
            copy_instruction("etc_motd--abc123", "/etc/motd", &None, &None, &None),
            "COPY etc_motd--abc123 /etc/motd"
        );
    }

    #[test]
    fn copy_instruction_with_chmod_only() {
        assert_eq!(
            copy_instruction("src", "/dst", &None, &None, &Some("644".into())),
            "COPY --chmod=644 src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_owner_only() {
        assert_eq!(
            copy_instruction("src", "/dst", &Some("root".into()), &None, &None),
            "COPY --chown=root src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_group_only() {
        assert_eq!(
            copy_instruction("src", "/dst", &None, &Some("wheel".into()), &None),
            "COPY --chown=:wheel src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_owner_and_group() {
        assert_eq!(
            copy_instruction(
                "src",
                "/dst",
                &Some("root".into()),
                &Some("wheel".into()),
                &None,
            ),
            "COPY --chown=root:wheel src /dst"
        );
    }

    #[test]
    fn copy_instruction_with_all_flags() {
        assert_eq!(
            copy_instruction(
                "src",
                "/dst",
                &Some("user".into()),
                &Some("grp".into()),
                &Some("755".into()),
            ),
            "COPY --chmod=755 --chown=user:grp src /dst"
        );
    }

    #[test]
    fn copy_instruction_dst_with_spaces_uses_quoted_form() {
        assert_eq!(
            copy_instruction("src", "/path with spaces", &None, &None, &None),
            r#"COPY ["src", "/path with spaces"]"#
        );
    }

    #[test]
    fn copy_instruction_dst_with_spaces_and_flags() {
        assert_eq!(
            copy_instruction(
                "src",
                "/dst file",
                &Some("root".into()),
                &None,
                &Some("644".into()),
            ),
            r#"COPY --chmod=644 --chown=root ["src", "/dst file"]"#
        );
    }

    // --- format_chown / format_chown_opt ---

    #[test]
    fn chown_owner_and_group() {
        assert_eq!(
            format_chown(&Some("root".into()), &Some("wheel".into())),
            "root:wheel"
        );
    }

    #[test]
    fn chown_owner_only() {
        assert_eq!(format_chown(&Some("root".into()), &None), "root");
    }

    #[test]
    fn chown_group_only() {
        assert_eq!(format_chown(&None, &Some("wheel".into())), ":wheel");
    }

    #[test]
    fn chown_opt_both_none_is_none() {
        assert_eq!(format_chown_opt(&None, &None), None);
    }

    #[test]
    fn chown_opt_with_owner_is_some() {
        assert!(format_chown_opt(&Some("root".into()), &None).is_some());
    }

    // --- Deserialization ---

    #[test]
    fn default_is_empty() {
        let field = FilesField::default();
        assert!(field.sources().is_empty());
        assert_eq!(field.value().unwrap().len(), 0);
    }

    #[test]
    fn null_deserializes_to_default() {
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let field: FilesField = serde_saphyr::from_str("~").unwrap();
        assert_eq!(field.value().unwrap().len(), 0);
    }

    #[test]
    fn deserialize_content_entry() {
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let field: FilesField =
            serde_saphyr::from_str("/etc/motd:\n  content: hello world\n").unwrap();
        let mut entries = field.value().unwrap();
        assert_eq!(entries.len(), 1);
        let (target, entry) = entries.remove(0);
        assert_eq!(target, "/etc/motd");
        assert!(matches!(&entry.source, FileSource::Content(s) if s == "hello world"));
        assert!(entry.owner.is_none());
        assert!(entry.group.is_none());
        assert!(entry.chmod.is_none());
    }

    #[test]
    fn deserialize_content_entry_with_metadata() {
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let field: FilesField = serde_saphyr::from_str(
            "/etc/motd:\n  content: hello\n  owner: root\n  group: wheel\n  chmod: '644'\n",
        )
        .unwrap();
        let mut entries = field.value().unwrap();
        let (_, entry) = entries.remove(0);
        assert_eq!(entry.owner.as_deref(), Some("root"));
        assert_eq!(entry.group.as_deref(), Some("wheel"));
        assert_eq!(entry.chmod.as_deref(), Some("644"));
    }

    #[test]
    fn deserialize_symlink_entry() {
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let field: FilesField =
            serde_saphyr::from_str("/usr/bin/sh:\n  symlink: /usr/bin/bash\n").unwrap();
        let mut entries = field.value().unwrap();
        let (target, entry) = entries.remove(0);
        assert_eq!(target, "/usr/bin/sh");
        assert!(matches!(&entry.source, FileSource::Symlink(s) if s == "/usr/bin/bash"));
    }

    #[test]
    fn deserialize_path_entry_resolves_relative_to_receipt() {
        let _guard = SourcePathGuard::push_path(p("/some/dir/receipt.yaml"));
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
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let result: Result<FilesField, _> =
            serde_saphyr::from_str("/etc/motd:\n  content: hello\n  symlink: /other\n");
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_missing_source_is_error() {
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let result: Result<FilesField, _> = serde_saphyr::from_str("/etc/motd:\n  owner: root\n");
        assert!(result.is_err());
    }

    #[test]
    fn merge_combines_entries_from_both() {
        let _guard1 = SourcePathGuard::push_path(p("/a.yaml"));
        let a: FilesField = serde_saphyr::from_str("/etc/a:\n  content: aaa\n").unwrap();
        let _guard2 = SourcePathGuard::push_path(p("/b.yaml"));
        let b: FilesField = serde_saphyr::from_str("/etc/b:\n  content: bbb\n").unwrap();
        let merged = a.merge(b);
        let entries = merged.value().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "/etc/a");
        assert_eq!(entries[1].0, "/etc/b");
    }

    #[test]
    fn duplicate_target_across_sources_is_conflict() {
        let _guard1 = SourcePathGuard::push_path(p("/a.yaml"));
        let a: FilesField = serde_saphyr::from_str("/etc/same:\n  content: aaa\n").unwrap();
        let _guard2 = SourcePathGuard::push_path(p("/b.yaml"));
        let b: FilesField = serde_saphyr::from_str("/etc/same:\n  content: bbb\n").unwrap();
        let merged = a.merge(b);
        assert!(matches!(merged.value(), Err(ReceiptError::FieldConflict)));
    }
}
