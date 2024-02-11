{
  description = "Wayland locker with some features";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.lib.${system};
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        llvm = pkgs.llvmPackages_17;

        # Just the introspection from fprintd
        fprintd-interfaces = pkgs.stdenv.mkDerivation rec {
          pname = "fprintd-interfaces";
          version = "1.94.2";
          src = pkgs.fetchFromGitLab {
            domain = "gitlab.freedesktop.org";
            owner = "libfprint";
            repo = "fprintd";
            rev = "v${version}";
            sha256 = "sha256-ePhcIZyXoGr8XlBuzKjpibU9D/44iCXYBlpVR9gcswQ=";
          };

          # There is a mistake in Device.xml. Doc tag references non-existing entity
          # It is patched out so codegen will work
          patches = [ ./fprintd.patch ];

          installPhase = ''
            mkdir -p $out
            cp src/net.reactivated.Fprint.Device.xml $out
            cp src/net.reactivated.Fprint.Manager.xml $out
          '';
        };

        commonEnv = {
          # Help bindgen find libclang.so
          LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
          # Help bindgen find headers
          CPATH = lib.strings.concatStringsSep ":" [
            "${pkgs.pam}/include"
            "${pkgs.glibc.dev}/include"
            "${llvm.libclang.lib}/lib/clang/17/include"
          ];

          FPRINT_DEVICE_XML = "${fprintd-interfaces}/net.reactivated.Fprint.Device.xml";
          FPRINT_MANAGER_XML = "${fprintd-interfaces}/net.reactivated.Fprint.Manager.xml";
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            libgcc
            libxkbcommon
            pam
            dbus
            fprintd
          ];

          runtimeDependencies = with pkgs; [
            wayland # sctk loads libwayland during runtime
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            autoPatchelfHook # Add runtimeDependencies to rpath
          ];
        } // commonEnv;

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

          my-crate-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        };

        packages.default = my-crate;

        apps.default = flake-utils.lib.mkApp {
          drv = my-crate;
        };

        devShells.default = craneLib.devShell ({
          checks = self.checks.${system};

          packages = with pkgs; [
            rust-analyzer
          ];

          # Convinient logging for develpment
          RUST_BACKTRACE = 1;
          RUST_LOG = "info";

          # When building with cargo autoPatchelf does not substitude
          # wayland library from rpath
          LD_LIBRARY_PATH = "${pkgs.wayland}/lib";
        } // commonEnv);
      });
}
