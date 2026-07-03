use azari::cli::generate::{
    schema::GenerateSchemaArgs,
    shell::{GenerateShellArgs, ShellKind},
};
use tempfile::tempdir;

// --- GenerateShellArgs ---

#[test]
fn generate_shell_single_writes_file() {
    let dir = tempdir().unwrap();

    let args = GenerateShellArgs {
        shell: ShellKind::Bash,
        path: Some(dir.path().to_path_buf()),
        install: false,
    };
    args.run().unwrap();

    let file = dir.path().join("azari.bash");
    assert!(file.exists());
    assert!(!std::fs::read(file).unwrap().is_empty());
}

#[test]
fn generate_shell_explicit_output_path() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("my-completions.fish");

    let args = GenerateShellArgs {
        shell: ShellKind::Fish,
        path: Some(out.clone()),
        install: false,
    };
    args.run().unwrap();

    assert!(out.exists());
    assert!(!std::fs::read(out).unwrap().is_empty());
}

#[test]
fn generate_shell_all_writes_each_shell_file() {
    let dir = tempdir().unwrap();

    let args = GenerateShellArgs {
        shell: ShellKind::All,
        path: Some(dir.path().to_path_buf()),
        install: false,
    };
    args.run().unwrap();

    for filename in ["azari.bash", "_azari", "azari.fish", "azari.nu"] {
        let file = dir.path().join(filename);
        assert!(file.exists(), "{filename} should exist");
        assert!(
            !std::fs::read(&file).unwrap().is_empty(),
            "{filename} should not be empty"
        );
    }
}

// --- GenerateSchemaArgs ---

#[test]
fn generate_schema_writes_valid_json() {
    let dir = tempdir().unwrap();

    let args = GenerateSchemaArgs {
        path: Some(dir.path().to_path_buf()),
        install: false,
    };
    args.run().unwrap();

    let file = dir.path().join("schema.json");
    assert!(file.exists());

    let content = std::fs::read_to_string(file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn generate_schema_explicit_output_path() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("my-schema.json");

    let args = GenerateSchemaArgs {
        path: Some(out.clone()),
        install: false,
    };
    args.run().unwrap();

    assert!(out.exists());
    let content = std::fs::read_to_string(out).unwrap();
    serde_json::from_str::<serde_json::Value>(&content).unwrap();
}

#[test]
fn generate_schema_contains_schema_version() {
    let dir = tempdir().unwrap();

    let args = GenerateSchemaArgs {
        path: Some(dir.path().join("schema.json")),
        install: false,
    };
    args.run().unwrap();

    let content = std::fs::read_to_string(dir.path().join("schema.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    // Draft-07 schema should have a $schema field
    assert!(parsed.get("$schema").is_some());
}
