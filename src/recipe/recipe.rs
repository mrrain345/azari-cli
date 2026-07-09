use std::collections::HashSet;
use std::path::{Path, PathBuf};

use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, BuildError, Builder};
use crate::recipe::RecipeField;
use crate::recipe::{RecipeError, fields::*, path::SourcePathGuard};

#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct Recipe {
    pub import: ImportField,

    pub distro: DistroField,
    pub image: ImageField,
    pub from: FromField,
    pub name: NameField,
    pub hostname: HostnameField,
    pub users: UsersField,
    pub files: FilesField,
    pub preinstall: PreinstallField,
    pub packages: PackagesField,
    pub systemd: SystemdField,
    pub postinstall: PostinstallField,
}

impl Build for Recipe {
    fn build(self, builder: &mut Builder) -> Result<(), BuildError> {
        self.distro.build(builder)?;
        self.from.build(builder)?;
        self.image.build(builder)?;
        self.name.build(builder)?;
        self.hostname.build(builder)?;
        self.users.build(builder)?;
        self.files.build(builder)?;
        self.preinstall.build(builder)?;
        self.packages.build(builder)?;
        self.systemd.build(builder)?;
        self.postinstall.build(builder)?;

        Ok(())
    }
}

impl Recipe {
    fn error(&self) -> Option<RecipeError> {
        let errors: Vec<RecipeError> = vec![
            self.import.error(),
            self.distro.error(),
            self.from.error(),
            self.image.error(),
            self.name.error(),
            self.hostname.error(),
            self.users.error(),
            self.files.error(),
            self.preinstall.error(),
            self.packages.error(),
            self.systemd.error(),
            self.postinstall.error(),
        ]
        .into_iter()
        .flatten()
        .collect();

        collapse_errors(errors)
    }

    pub fn from_file(path: &Path) -> Result<Self, RecipeError> {
        let mut seen = HashSet::new();
        let (recipe, mut errors) = Self::from_file_inner(path, &mut seen);

        if let Some(error) = recipe.as_ref().and_then(|r| r.error()) {
            errors.push(error);
        }

        if let Some(error) = collapse_errors(errors) {
            Err(error)
        } else {
            Ok(recipe.unwrap_or_default())
        }
    }

    fn from_file_inner(
        path: &Path,
        seen: &mut HashSet<PathBuf>,
    ) -> (Option<Self>, Vec<RecipeError>) {
        let canonical = match path.canonicalize() {
            Ok(path) => path,
            Err(source) => return (None, vec![io_error(path, source)]),
        };

        if !seen.insert(canonical.clone()) {
            return (Some(Self::default()), Vec::new());
        }

        let (mut current, mut errors) = Self::parse_single(&canonical);
        let mut base = Self::default();

        if let Some(current) = current.as_mut() {
            for next in current.import.take_imports() {
                let (imported, imported_errors) = Self::from_file_inner(&next, seen);
                errors.extend(imported_errors);

                if let Some(imported) = imported {
                    base.merge(imported);
                }
            }

            base.merge(std::mem::take(current));
        }

        (Some(base), errors)
    }

    fn parse_single(path: &Path) -> (Option<Self>, Vec<RecipeError>) {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(source) => return (None, vec![io_error(path, source)]),
        };

        let _guard = SourcePathGuard::push_path(path.into());

        match serde_saphyr::from_reader(file) {
            Ok(recipe) => (Some(recipe), Vec::new()),
            Err(source) => (
                None,
                vec![RecipeError::Parse {
                    path: path.to_path_buf(),
                    source: source.into(),
                }],
            ),
        }
    }
}

fn io_error(path: &Path, source: std::io::Error) -> RecipeError {
    RecipeError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn collapse_errors(mut errors: Vec<RecipeError>) -> Option<RecipeError> {
    match errors.len() {
        0 => None,
        1 => errors.pop(),
        _ => Some(RecipeError::Aggregate(errors)),
    }
}
