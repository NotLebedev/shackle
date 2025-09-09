{
  description = "Wayland locker with some features";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib nixpkgs.legacyPackages.${system});
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        llvm = pkgs.llvmPackages_17;

        commonEnv = {
          # Help bindgen find libclang.so
          LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
          # Help bindgen find headers
          CPATH = lib.strings.concatStringsSep ":" [
            "${pkgs.pam}/include"
            "${pkgs.glibc.dev}/include"
            "${llvm.libclang.lib}/lib/clang/17/include"
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            libgcc
            libxkbcommon
            pam
            dbus
            gtk4
            glib
            gtk4-layer-shell
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        }
        // commonEnv;

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        shackle = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        checks = {
          inherit shackle;

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        packages.default = shackle;

        apps.default = flake-utils.lib.mkApp {
          drv = shackle;
        };

        devShells.default = craneLib.devShell (
          {
            checks = self.checks.${system};

            packages = with pkgs; [
              rust-analyzer
            ];

            # Convinient logging for develpment
            RUST_BACKTRACE = 1;
            RUST_LOG = "info";
          }
          // commonEnv
        );
      }
    );
}
