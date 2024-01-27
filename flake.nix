{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            libxkbcommon
            wayland
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            gobject-introspection
          ];
        };

        # See https://github.com/diwic/dbus-rs/tree/master/dbus-codegen
        dbus-codegen-rust = craneLib.buildPackage {
          src = craneLib.downloadCargoPackage {
            name = "dbus-codegen";
            version = "0.10.0";
            checksum = "sha256-p23DXOg+Tp+gibT6vmbHV7J71QTcIXnJegGzbT6HT7A=";
            source = "registry+https://github.com/rust-lang/crates.io-index";
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            dbus
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        my-crate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        checks = {
          inherit my-crate;

          my-crate-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          my-crate-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          my-crate-fmt = craneLib.cargoFmt {
            inherit src;
          };

          my-crate-deny = craneLib.cargoDeny {
            inherit src;
          };

          my-crate-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        };

        packages = {
          default = my-crate;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = my-crate;
        };

        devShells.default = craneLib.devShell {
          # checks = self.checks.${system};

          packages = with pkgs; [
            rust-analyzer
            dbus-codegen-rust
            pkg-config
            libxkbcommon
            wayland
          ];

          LD_LIBRARY_PATH = "${pkgs.wayland}/lib/";
        };
      });
}
