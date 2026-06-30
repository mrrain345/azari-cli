use std::path::PathBuf;

use azari::recipe::{Recipe, RecipeError, RecipeField};

fn recipes_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/recipes")
}

#[test]
fn load_fields_recipe() {
    let path = recipes_dir().join("fields-full.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert_eq!(recipe.distro.value().unwrap().as_deref(), Some("arch"));

    assert_eq!(
        recipe.from.value().unwrap().as_deref(),
        Some("arch-bootc:latest")
    );
    assert_eq!(recipe.name.value().unwrap().as_deref(), Some("Azari OS"));
    assert_eq!(recipe.hostname.value().unwrap().as_deref(), Some("azari"));
}

#[test]
fn load_partial_recipe() {
    let path = recipes_dir().join("fields-partial.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert_eq!(recipe.from.value().unwrap(), None);
    assert_eq!(recipe.name.value().unwrap().as_deref(), Some("Azari OS"));
    assert_eq!(recipe.hostname.value().unwrap(), None);
}

#[test]
fn load_empty_recipe() {
    let path = recipes_dir().join("empty.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert_eq!(recipe.from.value().unwrap(), None);
    assert_eq!(recipe.name.value().unwrap(), None);
    assert_eq!(recipe.hostname.value().unwrap(), None);
}

#[test]
fn missing_file_returns_io_error() {
    let path = recipes_dir().join("does-not-exist.yaml");
    let result = Recipe::from_file(&path);

    assert!(
        matches!(result, Err(RecipeError::Io { .. })),
        "expected an I/O error for a missing file, got: {:?}",
        result
    );
}

#[test]
fn parse_error_includes_file_path() {
    let path = recipes_dir().join("parse-error.yaml");
    let result = Recipe::from_file(&path);

    match result {
        Err(RecipeError::Parse { path: error_path, .. }) => {
            assert_eq!(error_path, path);
        }
        other => panic!("expected parse error for invalid yaml, got: {:?}", other),
    }
}

