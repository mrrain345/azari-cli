use std::path::PathBuf;

use azari::builder::{BuildError, Builder};
use azari::recipe::fields::FromValue;
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
        recipe.from.value().unwrap(),
        Some(FromValue::Image("arch-bootc:latest".to_string()))
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
        Err(RecipeError::Parse {
            path: error_path, ..
        }) => {
            assert_eq!(error_path, path);
        }
        other => panic!("expected parse error for invalid yaml, got: {:?}", other),
    }
}

// --- Multi-stage build (from: path) ---

#[test]
fn from_stage_path_parses_as_stage_variant() {
    let path = recipes_dir().join("from-stage/main.yaml");
    let recipe = Recipe::from_file(&path).unwrap();

    let base_path = recipes_dir()
        .join("from-stage/base.yaml")
        .canonicalize()
        .unwrap();

    assert_eq!(
        recipe.from.value().unwrap(),
        Some(FromValue::Stage(base_path))
    );
}

#[test]
fn from_stage_inherits_distro_from_sub_recipe() {
    let path = recipes_dir().join("from-stage/main.yaml");
    // main.yaml has no `distro` field; it should be resolved from base.yaml
    let recipe = Recipe::from_file(&path).expect("failed to load recipe");
    let builder = Builder::from_recipe(recipe).expect("failed to create builder from recipe");
    // If distro resolution succeeded, the builder was created without error.
    // Verify it produced two stages (base + main).
    let containerfile = builder.to_containerfile();
    assert!(
        containerfile.contains("FROM arch-bootc:latest AS stage_1"),
        "expected stage_1 from base.yaml, got:\n{containerfile}"
    );
    assert!(
        containerfile.contains("FROM stage_1 AS stage_2"),
        "expected stage_2 referencing stage_1, got:\n{containerfile}"
    );
}

#[test]
fn from_stage_distro_conflict_returns_error() {
    let path = recipes_dir().join("from-stage-conflict/main.yaml");
    let result = Builder::from_recipe(Recipe::from_file(&path).unwrap());
    assert!(
        matches!(result, Err(BuildError::DistroConflict { .. })),
        "expected DistroConflict error, got: {:?}",
        result
    );
}
