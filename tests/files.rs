use std::path::PathBuf;

use azari_cli::builder::{BuildDir, Builder};
use azari_cli::receipt::{Receipt, ReceiptError, ReceiptField};

fn files_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/receipts/files")
}

// --- Field value tests ---

#[test]
fn load_content_files_field() {
    let path = files_dir().join("content.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let entries = receipt.files.value().unwrap();

    assert_eq!(entries.len(), 2);

    let (target, entry) = &entries[0];
    assert_eq!(target, "/etc/motd");
    assert!(entry.owner.is_none());
    assert!(entry.group.is_none());
    assert!(entry.chmod.is_none());

    let (target, entry) = &entries[1];
    assert_eq!(target, "/etc/sysconfig");
    assert_eq!(entry.owner.as_deref(), Some("root"));
    assert_eq!(entry.group.as_deref(), Some("wheel"));
    assert_eq!(entry.chmod.as_deref(), Some("644"));
}

#[test]
fn load_symlink_files_field() {
    use azari_cli::receipt::fields::FileSource;

    let path = files_dir().join("symlink.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let mut entries = receipt.files.value().unwrap();

    assert_eq!(entries.len(), 2);

    let (target, entry) = entries.remove(0);
    assert_eq!(target, "/usr/bin/sh");
    assert!(matches!(entry.source, FileSource::Symlink(ref s) if s == "/usr/bin/bash"));

    let (target, entry) = entries.remove(0);
    assert_eq!(target, "/tmp/owned-link");
    assert!(matches!(entry.source, FileSource::Symlink(ref s) if s == "/real/path"));
    assert_eq!(entry.owner.as_deref(), Some("user"));
    assert_eq!(entry.group.as_deref(), Some("grp"));
}

#[test]
fn load_path_files_field() {
    use azari_cli::receipt::fields::FileSource;

    let path = files_dir().join("path.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let mut entries = receipt.files.value().unwrap();

    assert_eq!(entries.len(), 1);
    let (target, entry) = entries.remove(0);
    assert_eq!(target, "/etc/builder");
    assert!(matches!(&entry.source, FileSource::Path(p) if p.ends_with("builder.yaml")));
}

// --- Build tests ---

/// Returns the path of the first file in `dir` whose name starts with `prefix`,
/// or `None` if no such file exists.
fn find_build_file(dir: &std::path::Path, prefix: &str) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .map(|e| e.path())
}

#[test]
fn build_content_file_writes_to_builddir() {
    let path = files_dir().join("content.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let dir_path = build_dir.path().to_owned();
    let _builder = Builder::from_receipt(receipt, build_dir, None).unwrap();

    let motd = find_build_file(&dir_path, "etc_motd--").expect("etc_motd-- file not found");
    assert_eq!(std::fs::read_to_string(&motd).unwrap(), "Hello from Azari");

    let cfg =
        find_build_file(&dir_path, "etc_sysconfig--").expect("etc_sysconfig-- file not found");
    assert_eq!(std::fs::read_to_string(&cfg).unwrap(), "[config]");
}

#[test]
fn build_content_file_emits_copy_instructions() {
    let path = files_dir().join("content.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let builder = Builder::from_receipt(receipt, build_dir, None).unwrap();
    let cf = builder.to_containerfile();

    assert!(
        cf.lines()
            .any(|l| l.starts_with("COPY ") && l.ends_with(" /etc/motd")),
        "expected simple COPY to /etc/motd in containerfile:\n{cf}"
    );
    assert!(
        cf.lines()
            .any(|l| l.starts_with("COPY --chmod=644 --chown=root:wheel ")
                && l.ends_with(" /etc/sysconfig")),
        "expected COPY with chmod+chown to /etc/sysconfig in containerfile:\n{cf}"
    );
}

#[test]
fn build_symlink_emits_run_ln_instruction() {
    let path = files_dir().join("symlink.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let builder = Builder::from_receipt(receipt, build_dir, None).unwrap();
    let cf = builder.to_containerfile();

    assert!(
        cf.contains("RUN ln -s '/usr/bin/bash' '/usr/bin/sh'"),
        "containerfile:\n{cf}"
    );
}

#[test]
fn build_symlink_with_owner_and_group_emits_chown() {
    let path = files_dir().join("symlink.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let builder = Builder::from_receipt(receipt, build_dir, None).unwrap();
    let cf = builder.to_containerfile();

    assert!(
        cf.contains("RUN ln -s '/real/path' '/tmp/owned-link'"),
        "containerfile:\n{cf}"
    );
    assert!(
        cf.contains("RUN chown -h user:grp '/tmp/owned-link'"),
        "containerfile:\n{cf}"
    );
}

#[test]
fn build_path_file_copies_to_builddir() {
    let path = files_dir().join("path.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let dir_path = build_dir.path().to_owned();
    let _builder = Builder::from_receipt(receipt, build_dir, None).unwrap();

    find_build_file(&dir_path, "etc_builder--").expect("etc_builder-- file not found");
}

#[test]
fn build_path_file_emits_copy_instruction() {
    let path = files_dir().join("path.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let builder = Builder::from_receipt(receipt, build_dir, None).unwrap();
    let cf = builder.to_containerfile();

    assert!(
        cf.lines()
            .any(|l| l.starts_with("COPY ") && l.ends_with(" /etc/builder")),
        "expected COPY to /etc/builder in containerfile:\n{cf}"
    );
}

// --- Import / merge tests ---

#[test]
fn files_from_imported_receipt_are_merged() {
    let path = files_dir().join("import-root.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let entries = receipt.files.value().unwrap();

    assert_eq!(entries.len(), 2);
    let targets: Vec<&str> = entries.iter().map(|(t, _)| t.as_str()).collect();
    assert!(
        targets.contains(&"/etc/base-file"),
        "missing base file, got: {targets:?}"
    );
    assert!(
        targets.contains(&"/etc/root-file"),
        "missing root file, got: {targets:?}"
    );
}

#[test]
fn duplicate_file_path_across_imports_returns_error() {
    let path = files_dir().join("conflict-root.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let result = receipt.files.value();

    assert!(
        matches!(result, Err(ReceiptError::FieldConflict)),
        "expected FieldConflict, got: {:?}",
        result
    );
}

// --- Spaces in paths ---

#[test]
fn spaces_in_target_produce_safe_build_dir_filename() {
    let path = files_dir().join("spaces.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let dir_path = build_dir.path().to_owned();
    let _builder = Builder::from_receipt(receipt, build_dir, None).unwrap();

    // The build-dir file must not have spaces in its name.
    let file = find_build_file(&dir_path, "etc_path_with_spaces--")
        .expect("etc_path_with_spaces-- file not found");
    assert!(!file.file_name().unwrap().to_string_lossy().contains(' '));
}

#[test]
fn spaces_in_content_target_use_quoted_copy_instruction() {
    let path = files_dir().join("spaces.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let builder = Builder::from_receipt(receipt, build_dir, None).unwrap();
    let cf = builder.to_containerfile();

    // COPY destination with spaces must use the JSON array form.
    assert!(
        cf.lines()
            .any(|l| l.starts_with("COPY ") && l.contains("[\"")),
        "expected JSON-array COPY for destination with spaces in containerfile:\n{cf}"
    );
}

#[test]
fn spaces_in_symlink_paths_are_shell_quoted() {
    let path = files_dir().join("spaces.yaml");
    let receipt = Receipt::from_file(&path).unwrap();
    let build_dir = BuildDir::temp().unwrap();
    let builder = Builder::from_receipt(receipt, build_dir, None).unwrap();
    let cf = builder.to_containerfile();

    assert!(
        cf.contains("RUN ln -s '/real/target with spaces' '/usr/link with spaces'"),
        "expected shell-quoted RUN ln -s in containerfile:\n{cf}"
    );
}
