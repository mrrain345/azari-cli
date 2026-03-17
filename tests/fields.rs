use std::path::PathBuf;

use azari_cli::receipt::{Receipt, ReceiptField};

fn receipts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/receipts")
}

#[test]
fn load_fields_receipt() {
    let path = receipts_dir().join("fields-full.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert_eq!(receipt.distro.value().unwrap().as_deref(), Some("arch"));

    assert_eq!(
        receipt.from.value().unwrap().as_deref(),
        Some("arch-bootc:latest")
    );
    assert_eq!(receipt.name.value().unwrap().as_deref(), Some("Azari OS"));
    assert_eq!(receipt.hostname.value().unwrap().as_deref(), Some("azari"));
}

#[test]
fn load_partial_receipt() {
    let path = receipts_dir().join("fields-partial.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert_eq!(receipt.from.value().unwrap(), None);
    assert_eq!(receipt.name.value().unwrap().as_deref(), Some("Azari OS"));
    assert_eq!(receipt.hostname.value().unwrap(), None);
}

#[test]
fn load_empty_receipt() {
    let path = receipts_dir().join("empty.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    assert_eq!(receipt.from.value().unwrap(), None);
    assert_eq!(receipt.name.value().unwrap(), None);
    assert_eq!(receipt.hostname.value().unwrap(), None);
}

#[test]
fn missing_file_returns_io_error() {
    use azari_cli::receipt::ReceiptError;

    let path = receipts_dir().join("does-not-exist.yaml");
    let result = Receipt::from_file(&path);

    assert!(
        matches!(result, Err(ReceiptError::Io(_))),
        "expected an I/O error for a missing file, got: {:?}",
        result
    );
}

#[test]
fn full_receipt_source_paths_are_set() {
    let path = receipts_dir().join("fields-full.yaml");
    let receipt = Receipt::from_file(&path).unwrap();

    let expected_path = path.canonicalize().ok();

    let from_path = receipt
        .from
        .sources()
        .first()
        .map(|p| p.canonicalize().ok());
    assert_eq!(from_path, Some(expected_path.clone()));

    let name_path = receipt
        .name
        .sources()
        .first()
        .map(|p| p.canonicalize().ok());
    assert_eq!(name_path, Some(expected_path.clone()));

    let hostname_path = receipt
        .hostname
        .sources()
        .first()
        .map(|p| p.canonicalize().ok());
    assert_eq!(hostname_path, Some(expected_path));
}
