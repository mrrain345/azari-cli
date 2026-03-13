use std::path::PathBuf;

use azari_cli::receipt::{Receipt, ReceiptError, ReceiptField};

fn receipts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/receipts")
}

#[test]
fn import_merges_fields_and_lists() {
    let path = receipts_dir().join("imports/root-imports.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert_eq!(
        receipt.from.value().unwrap().map(String::as_str),
        Some("root-image:latest")
    );
    assert_eq!(
        receipt.name.value().unwrap().map(String::as_str),
        Some("Root Name")
    );
    assert_eq!(
        receipt.hostname.value().unwrap().map(String::as_str),
        Some("root-host")
    );

    let packages: Vec<&str> = receipt
        .packages
        .value()
        .unwrap()
        .into_iter()
        .map(String::as_str)
        .collect();

    assert_eq!(
        packages,
        vec!["base-pkg-1", "base-pkg-2", "extra-pkg", "root-pkg"]
    );
}

#[test]
fn duplicate_import_is_ignored() {
    let path = receipts_dir().join("imports/root-duplicate.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert_eq!(
        receipt.from.value().unwrap().map(String::as_str),
        Some("dup-root-image")
    );
    assert_eq!(
        receipt.name.value().unwrap().map(String::as_str),
        Some("Dup Root")
    );

    let packages: Vec<&str> = receipt
        .packages
        .value()
        .unwrap()
        .into_iter()
        .map(String::as_str)
        .collect();

    assert_eq!(
        packages,
        vec!["base-pkg-1", "base-pkg-2", "dup-mid-pkg", "dup-root-pkg"]
    );
}

#[test]
fn circular_imports_do_not_recurse_forever() {
    let path = receipts_dir().join("imports/cycle-a.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert_eq!(
        receipt.from.value().unwrap().map(String::as_str),
        Some("cycle-a-image")
    );
    assert_eq!(
        receipt.name.value().unwrap().map(String::as_str),
        Some("Cycle A")
    );

    let packages: Vec<&str> = receipt
        .packages
        .value()
        .unwrap()
        .into_iter()
        .map(String::as_str)
        .collect();

    assert_eq!(packages, vec!["cycle-b-pkg", "cycle-a-pkg"]);
}

#[test]
fn import_pending_is_empty_after_full_load() {
    let path = receipts_dir().join("imports/root-duplicate.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert!(
        receipt.import.value().unwrap().is_empty(),
        "import pending list should be empty after full load"
    );
}

#[test]
fn missing_import_propagates_io_error() {
    let path = receipts_dir().join("imports/root-missing-import.yaml");
    let result = Receipt::from_file(&path);

    assert!(
        matches!(result, Err(ReceiptError::Parse(_))),
        "expected parse error for missing imported receipt, got: {:?}",
        result
    );
}
