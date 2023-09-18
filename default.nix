{ pkgs ? import <nixpkgs> {} }:
let
  unstable = import <nixos-unstable> { config = { allowUnfree = true; }; };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ unstable.rustc unstable.cargo unstable.gcc unstable.rustfmt unstable.clippy libiconv];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  NIX_ENFORCE_PURITY = 0;
}
