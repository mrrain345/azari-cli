use crate::builder::Builder;
use crate::distro::UserConfig;

/// Shared implementation of user account provisioning.
///
/// Logic is identical across all supported distros — call this from every
/// [`DistroOps::add_user`] implementation rather than duplicating it.
///
/// Pushes the Containerfile `RUN` instructions needed to create the account:
///
/// * A `useradd -m [options] <username>` instruction.
/// * If `password` is `None`, a `passwd -d <username>` instruction to allow
///   passwordless login. When a password is provided it is passed directly to
///   `useradd -p`; note that `-p` expects a **pre-hashed** value in crypt(3)
///   format — `useradd` does not hash plaintext strings itself.
pub fn add_user(builder: &mut Builder, config: &UserConfig) {
    let mut args: Vec<String> = vec!["useradd".into(), "-m".into()];

    if let Some(uid) = config.uid {
        args.extend(["-u".into(), uid.to_string()]);
    }
    if let Some(shell) = &config.shell {
        args.extend(["-s".into(), shell_quote(shell)]);
    }
    if let Some(home) = &config.home {
        args.extend(["-d".into(), shell_quote(home)]);
    }
    if let Some(fullname) = &config.fullname {
        args.extend(["-c".into(), shell_quote(fullname)]);
    }
    if let Some(password) = &config.password {
        args.extend(["-p".into(), shell_quote(password)]);
    }
    if !config.groups.is_empty() {
        args.extend(["-G".into(), config.groups.join(",")]);
    }
    args.push(shell_quote(&config.username));

    builder.push(format!("RUN {}", args.join(" ")));

    if config.password.is_none() {
        // Remove the locked/disabled state so the account can be used without a password.
        builder.push(format!("RUN passwd -d {}", shell_quote(&config.username)));
    }
}

/// Set the hostname via a `tmpfiles.d` configuration.
pub fn set_hostname(builder: &mut Builder, hostname: &str) {
    builder.push(format!(
        "RUN echo f+ /etc/hostname 0644 root root - {} > /usr/lib/tmpfiles.d/azari-hostname.conf",
        shell_quote(hostname)
    ));
}

/// Wraps `s` in POSIX single quotes, escaping any embedded single quotes.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
