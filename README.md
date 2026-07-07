# Azari

**Azari** is a declarative Linux bootable image builder. Describe your OS in a YAML config file, and Azari builds a bootable image with all your packages, files, users, and systemd units baked in.

## Installation

```sh
# Install the latest release from GitHub
sudo cargo install --locked --no-track --root /usr --git https://github.com/mrrain345/azari-cli.git
# Generate and install shell completions for bash, zsh, fish, and nushell
sudo azari generate shell all --install
```

## Example Config

Full documentation of the config file format is available in [docs/recipe.md](docs/recipe.md).

```yaml
distro: arch
image: ghcr.io/example/azari-arch
name: Azari Arch
hostname: my-machine

import:
  - common.yaml

packages:
  - git
  - neovim
  - podman

users:
  alice:
    fullname: Alice Smith
    shell: /bin/bash
    groups:
      - wheel
      - audio
      - video

files:
  /etc/motd:
    content: |
      Welcome to Azari Arch.
    chmod: 644
  /usr/local/bin/my-tool:
    path: ./assets/my-tool
    chmod: 755

systemd:
  - NetworkManager
  - cups

preinstall:
  - mkdir -p /opt/custom

postinstall:
  - rm -rf /tmp/* /var/tmp/* /var/cache/*
```

## Usage

The config file path can be set via the `AZARI_CONFIG` environment variable instead of passing `-c` every time.

```sh
# Build the image
azari build -c config.yaml

# Build with a version tag and push to the registry
azari build -c config.yaml --version 1.0.0 --push

# Show the status of the booted system
azari status

# Upgrade the currently installed system
azari upgrade

# Switch to a specific version
azari switch <version>

# Rollback to the previous deployment
azari rollback
```

## Installation of your system image

Make sure to back up any important data before installing, as this will overwrite the target device. Double-check the device path before running the command.

```sh
azari install --image ghcr.io/example/azari-arch /dev/sda
```

## VS Code Autocompletion

Generate the JSON schema for the config file and install it:

```sh
sudo azari generate schema --install
```

Install the [YAML](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml) extension, then add the following to `.vscode/settings.json` in your project:

```json
{
  "yaml.schemas": {
    "/usr/lib/azari/schema.json": "*.yaml"
  }
}
```

This enables autocompletion and validation for all YAML config files in your workspace.

## Documentation

- [Config Reference](docs/recipe.md)
