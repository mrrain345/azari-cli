use std::{env, path::PathBuf};

use clap::{Args, CommandFactory, ValueEnum};
use clap_complete::{Shell, generate};
use clap_complete_nushell::Nushell;

use crate::builder::BuildError;

use super::{super::Cli, resolve_path, write_output};

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    Nushell,
    All,
}

#[derive(Debug, Args)]
pub struct GenerateShellArgs {
    /// Shell to generate completions for
    pub shell: ShellKind,

    /// Output path (defaults to stdout; defaults to working directory for 'all')
    #[arg(value_name = "PATH", conflicts_with = "install")]
    pub path: Option<PathBuf>,

    /// Install completions to system path (requires root privileges)
    #[arg(short, long)]
    pub install: bool,
}

impl ShellKind {
    /// Returns a list of all shell variants except for `All`
    fn shells() -> Vec<&'static ShellKind> {
        ShellKind::value_variants()
            .iter()
            .filter(|s| !matches!(s, ShellKind::All))
            .collect::<Vec<_>>()
    }

    /// Returns a default filename for the completion file for this shell
    fn filename(&self) -> Option<&'static str> {
        match self {
            ShellKind::Bash => Some("azari.bash"),
            ShellKind::Zsh => Some("_azari"),
            ShellKind::Fish => Some("azari.fish"),
            ShellKind::Nushell => Some("azari.nu"),
            ShellKind::All => None,
        }
    }

    /// Returns the system install path for the completion file for this shell
    fn install_path(&self) -> Option<&'static str> {
        match self {
            ShellKind::Bash => Some("/usr/share/bash-completion/completions/azari"),
            ShellKind::Zsh => Some("/usr/share/zsh/site-functions/_azari"),
            ShellKind::Fish => Some("/usr/share/fish/vendor_completions.d/azari.fish"),
            ShellKind::Nushell => Some("/usr/share/nushell/vendor/autoload/azari.nu"),
            ShellKind::All => None,
        }
    }

    /// Generates the completion script for this shell
    ///
    /// # Panics
    /// Panics if `ShellKind::All` is used, as it does not correspond to a single shell.
    fn generate(&self) -> Vec<u8> {
        let mut cmd = Cli::command();
        let mut buf = Vec::new();

        match self {
            ShellKind::Bash => generate(Shell::Bash, &mut cmd, "azari", &mut buf),
            ShellKind::Zsh => generate(Shell::Zsh, &mut cmd, "azari", &mut buf),
            ShellKind::Fish => generate(Shell::Fish, &mut cmd, "azari", &mut buf),
            ShellKind::Nushell => generate(Nushell, &mut cmd, "azari", &mut buf),
            ShellKind::All => {
                panic!("ShellKind::All should not be used for generating completions")
            }
        }
        buf
    }
}

impl GenerateShellArgs {
    pub fn run(self) -> Result<(), BuildError> {
        match &self.shell {
            ShellKind::All => self.generate_all(),
            _ => self.generate_single(&self.shell),
        }
    }

    /// Generates completions for all supported shells
    fn generate_all(&self) -> Result<(), BuildError> {
        for shell in ShellKind::shells() {
            self.generate_single(shell)?;
        }
        Ok(())
    }

    /// Generates completions for a single shell
    fn generate_single(&self, shell: &ShellKind) -> Result<(), BuildError> {
        let path = self.get_shell_path(shell);
        let content = shell.generate();
        write_output(&content, path)
    }

    /// Returns the output path for the completion file for a given shell,
    /// taking into account the `--install` flag and the `PATH` option.
    fn get_shell_path(&self, shell: &ShellKind) -> Option<PathBuf> {
        // Ensure that `ShellKind::All` is not used here,
        // as it does not correspond to a single shell
        debug_assert_ne!(
            *shell,
            ShellKind::All,
            "ShellKind::All should not be used retrieving a path for a single shell"
        );

        if self.install {
            return Some(PathBuf::from(shell.install_path().unwrap()));
        }

        // If generating for all shells and no path is provided,
        // default to the current working directory
        let path = match self.shell {
            ShellKind::All if self.path.is_none() => env::current_dir().ok(),
            _ => self.path.clone(),
        };

        resolve_path(path.as_ref(), shell.filename().unwrap())
    }
}
