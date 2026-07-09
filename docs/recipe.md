# Config Reference

An azari config file is a YAML document that describes your Linux image. Every field is optional — only include what you need. Multiple config files can be composed together via [`import`](#import).

---

## Build order

Fields are applied in the following order, regardless of their order in the file:

1. [`import`](#import)
2. [`distro`](#distro)
3. [`image`](#image)
4. [`from`](#from)
5. [`name`](#name)
6. [`hostname`](#hostname)
7. [`users`](#users)
8. [`files`](#files)
9. [`preinstall`](#preinstall)
10. [`packages`](#packages)
11. [`systemd`](#systemd)
12. [`postinstall`](#postinstall)

---

## `import`

Import additional config files. Imported files are merged before the current file, so values in the current file always take precedence.

Paths may be absolute or relative to the current config file.

```yaml
import:
  - common.yaml
  - ../shared/base.yaml
```

---

## `distro`

Target Linux distribution. Selects distro-specific defaults such as the base image and package manager.

**Possible values:** `arch`, `debian`, `fedora`, `ubuntu`

```yaml
distro: arch
```

---

## `image`

Name of the output image, including the registry and repository.

```yaml
image: ghcr.io/username/my-system
```

---

## `from`

Set the base image for the current stage. Accepts either a container image reference or a path to another config file.

**Image reference** — overrides the distro's default base image. The image must be a valid bootc image for the selected distro. If omitted, the distro's default base image is used.

```yaml
from: ghcr.io/bootcrew/arch-bootc:latest
```

**Config file path** — enables multi-stage builds. The referenced config is built as a preceding stage and the current config starts from its output.

```yaml
from: ./base.yaml
```

The `distro` field must be consistent across all stages in a multi-stage build: it may be defined in at most one config (or set to the same value in all of them).

---

## `name`

Human-readable OS name.

```yaml
name: My Workstation
```

---

## `hostname`

Set the default hostname of the image.

```yaml
hostname: my-machine
```

---

## `users`

Users to create in the image. The key is the username; the value is the account settings. All fields are optional.

| Field      | Description                                                                                                                       |
| ---------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `fullname` | Full display name                                                                                                                 |
| `password` | Plaintext or hashed password. Generate a hash with `openssl passwd -6`. If omitted, the user allows for login without a password. |
| `uid`      | Numeric user ID                                                                                                                   |
| `shell`    | Login shell path                                                                                                                  |
| `home`     | Home directory path                                                                                                               |
| `groups`   | List of additional groups to add the user to                                                                                      |

> **Note:** The hashed password is stored in the image. Anyone with access to the image can potentially retrieve it.

```yaml
users:
  alice:
    fullname: Alice Smith
    password: "$6$rounds=4096$..."
    shell: /bin/bash
    groups:
      - wheel
      - audio
      - video
  bob:
    uid: 1001
    shell: /usr/bin/fish
    home: /home/bob
```

---

## `files`

Files, directories, or symlinks to place in the image. The key is the destination path inside the image.

Set exactly one of `content`, `path`, or `symlink` per entry.

| Field     | Description                                                                                          |
| --------- | ---------------------------------------------------------------------------------------------------- |
| `content` | Inline text to write to the destination path                                                         |
| `path`    | Path to a local source file or directory. Relative paths are resolved from the config file location. |
| `symlink` | Symlink target path inside the image                                                                 |
| `owner`   | File owner (for `content` and `path`)                                                                |
| `group`   | File group (for `content` and `path`)                                                                |
| `chmod`   | File mode, e.g. `644` or `755`                                                                       |

```yaml
files:
  /etc/motd:
    content: |
      Welcome to My System.
    owner: root
    group: root
    chmod: 644

  /usr/local/bin/my-tool:
    path: ./assets/my-tool
    chmod: 755

  /etc/localtime:
    symlink: /usr/share/zoneinfo/UTC
```

---

## `preinstall`

Shell commands to execute at the **beginning** of the build, before packages are installed. Each entry becomes a separate `RUN` instruction.

```yaml
preinstall:
  - mkdir -p /opt/custom
  - curl -fsSL https://example.com/setup.sh | sh
```

---

## `packages`

Packages to install using the distro's package manager.

```yaml
packages:
  - git
  - neovim
  - podman
  - cargo
```

---

## `systemd`

Systemd units to enable or define. Supports two forms that can be combined across imported configs.

### Simple form

Enable existing units by name. Equivalent to `systemctl enable <name>`.

```yaml
systemd:
  - NetworkManager
  - cups
  - avahi-daemon
```

### Complex form

Define full unit files inline and/or enable existing units. The key is the unit name (without the `.service`, `.socket`, etc. extension — the type is inferred from the sections present).

```yaml
systemd:
  my-service:
    service:
      enabled: true
      unit:
        description: My Service
        after: network.target
      service:
        type: oneshot
        exec-start: /usr/bin/my-service
      install:
        wanted-by: multi-user.target
```

Set `user: true` to install as a systemd user unit (`systemctl --global enable`):

```yaml
systemd:
  my-user-service:
    user: true
    service:
      enabled: true
      unit:
        description: My User Service
        after: default.target
      service:
        type: simple
        exec-start: /usr/bin/my-user-service
      install:
        wanted-by: default.target
```

### Unit entry fields

| Field     | Description                                 |
| --------- | ------------------------------------------- |
| `user`    | Install as a user unit (`false` by default) |
| `service` | `.service` unit configuration (see below)   |
| `socket`  | `.socket` unit configuration (see below)    |
| `timer`   | `.timer` unit configuration (see below)     |
| `path`    | `.path` unit configuration (see below)      |
| `target`  | `.target` unit configuration (see below)    |

### Common unit fields

All unit types share a common structure:

| Field     | Description                                     |
| --------- | ----------------------------------------------- |
| `enabled` | Whether to enable the unit (`systemctl enable`) |
| `unit`    | `[Unit]` section (see below)                    |
| `install` | `[Install]` section (see below)                 |

Plus a type-specific section (`service:`, `socket:`, `timer:`, `path:`).

Each section allows arbitrary fields to be passed through to the unit file.
See the systemd documentation for a complete list of available directives:

- [Unit documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html)
- [Service documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html)
- [Socket documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd.socket.html)
- [Timer documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd.timer.html)
- [Path documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd.path.html)
- [Target documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd.target.html)

### `[Unit]` section

| Field           | Description                |
| --------------- | -------------------------- |
| `description`   | Human-readable description |
| `after`         | Order after these units    |
| `wants`         | Weak dependencies          |
| `requires`      | Hard dependencies          |
| `documentation` | Documentation URI          |

### `[Install]` section

| Field       | Description                                   |
| ----------- | --------------------------------------------- |
| `wanted-by` | Targets that pull this unit in when enabled   |
| `also`      | Additional units to enable alongside this one |

### `[Service]` section

| Field               | Description                                                     |
| ------------------- | --------------------------------------------------------------- |
| `type`              | Activation type: `simple`, `oneshot`, `notify`, `forking`, etc. |
| `exec-start-pre`    | Commands run before the main process (list or string)           |
| `exec-start`        | Main process command(s) (list or string)                        |
| `exec-reload`       | Command run on `systemctl reload`                               |
| `restart`           | Restart policy: `always`, `on-failure`, `no`, etc.              |
| `restart-sec`       | Seconds between restarts                                        |
| `timeout-start-sec` | Seconds to wait for startup                                     |
| `environment`       | `KEY=VALUE` environment variables (list or string)              |

Any additional `[Service]` directives not listed above can be passed as extra fields and will be written verbatim.

### `[Socket]` section

| Field           | Description                                        |
| --------------- | -------------------------------------------------- |
| `listen-stream` | Filesystem path or address to listen on            |
| `socket-mode`   | Octal permission mode for the socket (e.g. `0660`) |
| `socket-user`   | User that owns the socket node                     |
| `socket-group`  | Group that owns the socket node                    |

### `[Timer]` section

| Field                  | Description                                                |
| ---------------------- | ---------------------------------------------------------- |
| `on-calendar`          | Calendar expression(s), e.g. `daily`                       |
| `on-boot-sec`          | Delay relative to system boot                              |
| `on-active-sec`        | Delay relative to timer activation                         |
| `on-start-sec`         | Delay relative to timer start                              |
| `on-unit-active-sec`   | Delay relative to the last activation of the linked unit   |
| `on-unit-inactive-sec` | Delay relative to the last deactivation of the linked unit |
| `persistent`           | Keep schedule across downtime                              |
| `randomized-delay-sec` | Add a random delay to spread load                          |

### `[Path]` section

| Field              | Description                                  |
| ------------------ | -------------------------------------------- |
| `path-exists`      | Trigger when a path appears                  |
| `path-exists-glob` | Trigger when a glob matches an existing path |
| `path-changed`     | Trigger when a path's metadata changes       |
| `path-modified`    | Trigger when a path's contents change        |

---

## `postinstall`

Shell commands to execute at the **end** of the build, after packages and files are placed. Each entry becomes a separate `RUN` instruction.

```yaml
postinstall:
  - rm -rf /tmp/* /var/tmp/* /var/cache/*
```
