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

        # Registry crates fetched at eval time from Cargo.lock checksums, so checks need no network.
        vendored = pkgs.rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };

        # The checks build the whole workspace, src-tauri included, so they need the Tauri libraries too.
        cargoCheck = name: cmd:
          pkgs.runCommand name
            {
              src = ./.;
              nativeBuildInputs = [ toolchain pkgs.pkg-config ];
              buildInputs = tauriLibs;
            }
            ''
              cp -r $src source && chmod -R u+w source && cd source
              # Cargo needs a writable home even when offline.
              export CARGO_HOME=$TMPDIR/cargo CARGO_TARGET_DIR=$TMPDIR/target
              # Point crates.io at the vendored copy: https://doc.rust-lang.org/cargo/reference/source-replacement.html
              mkdir -p $CARGO_HOME
              cat > $CARGO_HOME/config.toml <<EOF
              [source.crates-io]
              replace-with = "vendored"
              [source.vendored]
              directory = "${vendored}"
              EOF
              # tauri::generate_context! embeds ../ui/dist, which is built output and not in git.
              mkdir -p ui/dist
              ${cmd}
              touch $out
            '';

        # Only the manifest and lockfile feed the fetch, so UI source edits do not refetch.
        uiDeps = pkgs.fetchPnpmDeps {
          pname = "bd-ui";
          version = "0.1.0";
          src = pkgs.lib.fileset.toSource {
            root = ./ui;
            fileset = pkgs.lib.fileset.unions [ ./ui/package.json ./ui/pnpm-lock.yaml ];
          };
          fetcherVersion = 5;
          # Update when ui/pnpm-lock.yaml changes: set to pkgs.lib.fakeHash, run `nix build`, copy the
          # "got:" hash from the error.
          hash = "sha256-gaJDUvwKBZe5VHVXn8Dg8nNQAETNX1DMuihJUKfMCYs=";
        };

        # The pinned toolchain, not nixpkgs' own rustc, because specta needs Rust 1.93.
        rustPlatform = pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; };

        # cargo-tauri.hook runs `cargo tauri build --bundles deb` and installs the deb's contents
        # (binary, .desktop file, icons), so the package matches what the deb ships.
        biggus-darkus = rustPlatform.buildRustPackage {
          pname = "biggus-darkus";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildAndTestSubdir = "src-tauri";

          nativeBuildInputs = with pkgs; [
            cargo-tauri.hook
            pkg-config
            wrapGAppsHook3
            nodejs
            pnpm
            pnpmConfigHook
          ];
          buildInputs = tauriLibs ++ [ pkgs.glib-networking ];

          pnpmDeps = uiDeps;
          pnpmRoot = "ui";

          # Tauri runs beforeBuildCommand from a directory where `../ui` does not exist inside the nix
          # build, so build the UI here and skip that command.
          preBuild = "pnpm --dir ui build";
          tauriBuildFlags = [ "--config" ''{"build":{"beforeBuildCommand":""}}'' ];

          # The unit tests run in the checks; the package build only needs the binary.
          doCheck = false;

          # Wayland workaround, see the dev shell. A wrapper, because the workspace forbids unsafe_code
          # and so cannot call std::env::set_var.
          preFixup = ''
            gappsWrapperArgs+=(--set GDK_BACKEND x11)
          '';

          meta.mainProgram = "bd-app";
        };
      in
      {
        packages.default = biggus-darkus;
        apps.default = flake-utils.lib.mkApp { drv = biggus-darkus; };

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

          # On native Wayland, this WebKitGTK reads no screen resolution from GTK and scales the page
          # by -1/96, so the window shows a blank, unstyled page. XWayland reports a real one.
          GDK_BACKEND = "x11";
        };

        checks = {
          fmt = cargoCheck "bd-fmt" "cargo fmt --all --check";
          clippy = cargoCheck "bd-clippy" "cargo clippy --workspace --all-targets --offline -- -D warnings";
          test = cargoCheck "bd-test" "cargo test --workspace --offline";
        };
      });
}
