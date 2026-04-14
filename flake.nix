{
  description = "karukan - Japanese Input Method for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      mkPkgs = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      mkToolchain = pkgs: pkgs.rust-bin.stable.latest.default;

      mkCraneLib = pkgs:
        let
          toolchain = mkToolchain pkgs;
        in
        (crane.mkLib pkgs).overrideToolchain toolchain;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = mkPkgs system;
          craneLib = mkCraneLib pkgs;

          # Common args for Rust builds
          commonArgs = {
            pname = "karukan";
            version = "0.1.0";
            src = craneLib.cleanCargoSource ./.;

            nativeBuildInputs = with pkgs; [
              cmake
              pkg-config
              llvmPackages.libclang.lib
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

            # bindgen needs system headers (stdio.h etc.)
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.llvmPackages.libclang}/include -isystem ${pkgs.glibc.dev}/include";
          };

          # Build deps once, share across packages
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # karukan-cli binaries (karukan-server, karukan-dict, sudachi-dict, ajimee-bench)
          karukan-cli = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p karukan-cli";
          });

          # karukan-engine library (built as part of cli/im, exposed for testing)
          karukan-engine = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p karukan-engine";
          });

          # karukan-im shared library
          karukan-im-lib = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p karukan-im";

            # Install the cdylib
            postInstall = ''
              mkdir -p $out/lib
              find target/release -name 'libkarukan_im.so' -exec cp {} $out/lib/ \;
            '';
          });

          # fcitx5 addon (C++ wrapper around karukan-im)
          karukan-fcitx5 = pkgs.stdenv.mkDerivation {
            pname = "fcitx5-karukan";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = with pkgs; [
              cmake
              pkg-config
              extra-cmake-modules
            ];

            buildInputs = with pkgs; [
              fcitx5
              libxkbcommon
            ];

            # Skip the Rust build in CMake — use pre-built library from karukan-im-lib
            cmakeDir = "../karukan-im/fcitx5-addon";

            preConfigure = ''
              # Point CMake to the pre-built Rust library
              mkdir -p target/release
              cp ${karukan-im-lib}/lib/libkarukan_im.so target/release/
            '';

            cmakeFlags = [
              "-DCMAKE_INSTALL_PREFIX=${placeholder "out"}"
            ];

            # Patch CMakeLists.txt to skip cargo build
            postPatch = ''
              substituteInPlace karukan-im/fcitx5-addon/CMakeLists.txt \
                --replace-fail 'find_program(CARGO cargo REQUIRED)' '# cargo: skipped in Nix' \
                --replace-fail 'add_custom_target(karukan_rust_lib ALL' 'add_custom_target(karukan_rust_lib' \
                --replace-fail 'COMMAND ''${CARGO} build --release -p karukan-im' '# cargo build: skipped in Nix' \
                --replace-fail 'BYPRODUCTS "''${KARUKAN_RUST_LIB}"' '# byproducts: skipped in Nix'
            '';

            postInstall = ''
              # Install the Rust shared library alongside the addon
              cp ${karukan-im-lib}/lib/libkarukan_im.so $out/lib/fcitx5/
            '';
          };
        in
        {
          inherit karukan-cli karukan-engine karukan-fcitx5;
          default = karukan-cli;
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = mkPkgs system;
          toolchain = mkToolchain pkgs;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              toolchain
              cmake
              pkg-config
              extra-cmake-modules
              rust-analyzer
              llvmPackages.libclang.lib
            ];

            buildInputs = with pkgs; [
              fcitx5
              libxkbcommon
              openssl
              python314
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.llvmPackages.libclang}/include -isystem ${pkgs.glibc.dev}/include";
          };
        }
      );
    };
}
