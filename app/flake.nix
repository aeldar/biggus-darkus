{
  description = "Biggus Darkus app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # One source of truth for the Rust version: rust-toolchain.toml.
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Tauri v2 Linux libraries: https://v2.tauri.app/start/prerequisites/#linux
        tauriLibs = with pkgs; [
          webkitgtk_4_1
          gtk3
          libsoup_3
          librsvg
          openssl
          systemd.dev # libudev, needed by serialport
        ];

        # Rust-only checks need just the toolchain, so they stay fast and light.
        cargoCheck = name: cmd:
          pkgs.runCommand name
            {
              src = ./.;
              nativeBuildInputs = [ toolchain pkgs.pkg-config ];
            }
            ''
              cp -r $src source && chmod -R u+w source && cd source
              # Cargo needs a writable home even when offline.
              export CARGO_HOME=$TMPDIR/cargo CARGO_TARGET_DIR=$TMPDIR/target
              ${cmd}
              touch $out
            '';
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            pkgs.pkg-config
            pkgs.nodejs
            pkgs.pnpm
            pkgs.cargo-tauri
          ] ++ tauriLibs;

          # Tauri loads GTK/WebKit at run time, so the loader must find them.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath tauriLibs;
        };

        checks = {
          fmt = cargoCheck "bd-fmt" "cargo fmt --all --check";
          clippy = cargoCheck "bd-clippy" "cargo clippy --workspace --all-targets --offline -- -D warnings";
          test = cargoCheck "bd-test" "cargo test --workspace --offline";
        };
      });
}
