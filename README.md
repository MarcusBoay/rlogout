# rlogout
A rewrite of [wlogout](https://github.com/ArtsyMacaw/wlogout) in Rust and GTK4.
![showcase](imgs/showcase.png)

## Notable differences
- Window only appears on single screen, `-n, --no-span` option is gone as a result. This is because
  GTK4 disallows as wayland does not have global screen coordinates (which is what GTK4 is modelled
  off of)
- Layout file is named `layout.json` instead of `layout`
- Layout file must be a valid json file (put the original `layout` file in an array and separate
  each entry with a comma) (see [layout.json](layout.json))
- Config directory is `rlogout` instead of `wlogout` (example: `~/.config/rlogout`)

## Additional features
- optional `label_x_align`, and `label_y_align` to manipulate the position for the label of buttons
  (see [layout.json](imgs/showcase.png))

![label_positioning](imgs/label_positioning.png)

## Build
To install the dependencies, you would need to use [nix](https://nixos.org/download/). Once you have
that, run `nix-shell`. This will install and load all dependencies listed in `shell.nix`.
