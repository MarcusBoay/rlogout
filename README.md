# rlogout
A rewrite of [wlogout](https://github.com/ArtsyMacaw/wlogout) in Rust and GTK4.

## Notable differences
- Window only appears on single screen, `-n, --no-span` option is gone as a result. This is because
  GTK4 disallows as wayland does not have global screen coordinates (which is what GTK4 is modelled
  off of)
- Layout file is named `layout.json` instead of `layout`
- Layout file must be a valid json file (put the original `layout` file in an array and separate
  each entry with a comma)
- config directory is `rlogout` instead of `wlogout` (example: `~/.config/rlogout`)

## Build
To install the dependencies, you would need to use [nix](https://nixos.org/download/). Once you have
that, run `nix-shell`. This will install and load all dependencies listed in `shell.nix`.
