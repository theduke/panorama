# panorama

A notification daemon for Linux systems with a bare-bones desktop environment,
like i3 or sway.

## Features

- [x] Battery status notifications
- [ ] Internet offline/online notifications
- [ ] Full disk notifications
- [ ] disk mount/unmount notifications
- [ ] USB device attach/detach notifications

## Installation

### Nix(OS)

If you use nix or NixOS, panorama can easily be run through the official flake:

```
nix run github.com/theduke/panorama
```

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

By default panorama will search for a configuration file in `$HOME/.config/panorama/config.toml`.
You can customize the config file with `-c/--config <PATH>`.

To generate a default config, use `panorama --dump-default-config`.
