# panorama

A system status notification daemon for Linux systems.
Panorama will send notifications for various system events, and is primarily
useful for bare-bones desktop environments like i3 or sway.

## Features

- [x] Battery status notifications
- [ ] Internet offline/online notifications
- [ ] High disk usage warnings
- [ ] disk mount/unmount notifications
- [ ] USB device attach/detach notifications

## Installation

### Nix(OS)

If you use nix or NixOS, panorama can easily be run through the official flake:

```
nix run github.com/theduke/panorama
```

To install, you can use the flake as a dependency in your own system flake.nix.

**Note**: panorama will hopefully be upstreamed into nixpkgs soon.

### Install from source

If you have Rust installed, you can install easily install panorama from source:

```
git clone https://github.com/theduke/panorama
cd panorama
cargo install --path .
```

## Usage

Just run the `panorama` command.
This will start the daemon.

## Configuration

Panorama can be heavily customized through the configuration file.

The default file path is `$HOME/.config/panorama/config.toml`, and can
be customized with `-c/--config <PATH>`.

To generate a default config, use `panorama --dump-default-config`.

You can inspect the default file to get information about all the options.

The default config is also available online: [config.toml](./config.toml).
