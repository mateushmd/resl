{ pkgs ? import <nixpkgs> {} }:
let
  buildInputs = with pkgs; [
    # rust
    cargo
    clippy
    rustc
    rustfmt
    gcc
    gdb

    # bevy
    udev
    alsa-lib
    vulkan-loader
    wayland
    libxkbcommon
  ];
in
pkgs.mkShell {
  inherit buildInputs;

  nativeBuildInputs  = with pkgs; [
    pkg-config 
  ];
  
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
