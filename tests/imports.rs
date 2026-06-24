use std::path::PathBuf;

use azari::recipe::{Recipe, RecipeError, RecipeField};

fn recipes_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/recipes")
}

#[test]
fn import_merges_fields_and_lists() {
    let path = recipes_dir().join("imports/root-imports.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert_eq!(recipe.distro.value().unwrap().as_deref(), Some("arch"));

    assert_eq!(recipe.name.value().unwrap().as_deref(), Some("Root Name"));
    assert_eq!(
        recipe.hostname.value().unwrap().as_deref(),
        Some("root-host")
    );

    let packages: Vec<String> = recipe.packages.value().unwrap();

    assert_eq!(
        packages,
        &["base-pkg-1", "base-pkg-2", "extra-pkg", "root-pkg"]
    );
}

#[test]
fn duplicate_import_is_ignored() {
    let path = recipes_dir().join("imports/root-duplicate.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert_eq!(recipe.distro.value().unwrap().as_deref(), Some("arch"));
    assert_eq!(recipe.name.value().unwrap().as_deref(), Some("Dup Root"));

    let packages: Vec<String> = recipe.packages.value().unwrap();

    assert_eq!(
        packages,
        vec!["base-pkg-1", "base-pkg-2", "dup-mid-pkg", "dup-root-pkg"]
    );
}

#[test]
fn circular_imports_do_not_recurse_forever() {
    let path = recipes_dir().join("imports/cycle-a.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert_eq!(
        recipe.from.value().unwrap().as_deref(),
        Some("cycle-a-image")
    );
    assert_eq!(recipe.name.value().unwrap().as_deref(), Some("Cycle A"));

    let packages: Vec<String> = recipe.packages.value().unwrap();

    assert_eq!(packages, vec!["cycle-b-pkg", "cycle-a-pkg"]);
}

#[test]
fn import_pending_is_empty_after_full_load() {
    let path = recipes_dir().join("imports/root-duplicate.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    assert!(
        recipe.import.value().unwrap().is_empty(),
        "import pending list should be empty after full load"
    );
}

#[test]
fn missing_import_propagates_io_error() {
    let path = recipes_dir().join("imports/root-missing-import.yaml");
    let result = Recipe::from_file(&path);

    assert!(
        matches!(result, Err(RecipeError::Parse(_))),
        "expected parse error for missing imported recipe, got: {:?}",
        result
    );
}
