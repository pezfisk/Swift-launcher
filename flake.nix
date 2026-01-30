{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, naersk, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs
          {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

        rustToolChain = pkgs.rust-bin.stable.latest.default.override
          {
            extensions = [ "rust-src" "rust-analyzer" ];
            targets = [ "wasm32-wasip1" ];
          };

        naersk-lib = pkgs.callPackage naersk {
          cargo = rustToolChain;
          rustc = rustToolChain;
        };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [
            rustToolChain
            cargo-component
            cargo-bloat
            wasmtime
            rustfmt
            pre-commit
            rustPackages.clippy
            gcc
            libx11
            libGL
            qt6.qtbase
            pkg-config
            pam
            upx
            rust-analyzer
            lldb_20
            slint-lsp
            flatpak-builder
            flatpak-xdg-utils
            appstream
            act

            lazygit
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;

          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${ with pkgs; lib.makeLibraryPath [
                wayland
                libxkbcommon
                fontconfig
                pam
          ] }";
        };
      }
    );
}
