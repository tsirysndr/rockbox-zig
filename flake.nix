{
  description = "Rockbox Zig — dev environment for building rockboxd and the rockbox CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs    = import nixpkgs { inherit system overlays; };
        lib     = pkgs.lib;

        # ── Rust 1.95 stable ────────────────────────────────────────────────
        rustToolchain = pkgs.rust-bin.stable."1.95.0".default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        # ── Zig 0.16.0 (fetched from upstream; not yet in all nixpkgs pins) ─
        zigVersion = "0.16.0";

        # Map Nix system strings to the upstream tarball platform strings and
        # their sha256 checksums (from https://ziglang.org/download/0.16.0/index.json).
        zigBySystem = {
          "x86_64-linux"   = { plat = "x86_64-linux";  sha256 = "70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00"; };
          "aarch64-linux"  = { plat = "aarch64-linux"; sha256 = "ea4b09bfb22ec6f6c6ceac57ab63efb6b46e17ab08d21f69f3a48b38e1534f17"; };
          "x86_64-darwin"  = { plat = "x86_64-macos";  sha256 = "0387557ed1877bc6a2e1802c8391953baddba76081876301c522f52977b52ba7"; };
          "aarch64-darwin" = { plat = "aarch64-macos"; sha256 = "b23d70deaa879b5c2d486ed3316f7eaa53e84acf6fc9cc747de152450d401489"; };
        };

        zigInfo = zigBySystem.${system};

        zig = pkgs.stdenv.mkDerivation {
          pname   = "zig";
          version = zigVersion;
          src = pkgs.fetchurl {
            url    = "https://ziglang.org/download/${zigVersion}/zig-${zigInfo.plat}-${zigVersion}.tar.xz";
            sha256 = zigInfo.sha256;
          };
          dontConfigure = true;
          dontBuild     = true;
          installPhase  = ''
            mkdir -p $out/bin $out/lib
            cp -r lib $out/lib/zig
            cp zig  $out/bin/zig
          '';
          meta = with lib; {
            description = "Zig ${zigVersion} compiler and toolchain";
            homepage    = "https://ziglang.org";
            license     = licenses.mit;
            platforms   = builtins.attrNames zigBySystem;
          };
        };

        # ── Platform-specific packages ───────────────────────────────────────

        # Linux: ALSA (cpal / headless build), D-Bus, libunwind.
        linuxPkgs = lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          alsa-lib alsa-lib.dev
          dbus     dbus.dev
          libunwind libunwind.dev
        ]);

        # macOS: llvm-objcopy for the codec --redefine-sym step inside
        # scripts/build-headless.sh.  Use llvmPackages_18.llvm (not .bintools —
        # bintools wraps Apple's ld and requires the removed apple_sdk_11_0 stub).
        # macOS system frameworks (CoreAudio, AudioToolbox, …) are available
        # automatically through the Xcode/CLT SDK; no explicit Nix inputs needed.
        darwinPkgs = lib.optionals pkgs.stdenv.isDarwin (with pkgs; [
          llvmPackages_18.llvm  # provides llvm-objcopy for Mach-O codec builds
        ]);

        # ── PKG_CONFIG_PATH / LD_LIBRARY_PATH helpers ────────────────────────

        pkgConfigDirs = lib.concatStringsSep ":" (
          [
            "${pkgs.SDL2.dev}/lib/pkgconfig"
            "${pkgs.freetype.dev}/lib/pkgconfig"
            "${pkgs.zlib.dev}/lib/pkgconfig"
            "${pkgs.libusb1.dev}/lib/pkgconfig"
          ]
          ++ lib.optionals pkgs.stdenv.isLinux [
            "${pkgs.alsa-lib.dev}/lib/pkgconfig"
            "${pkgs.dbus.dev}/lib/pkgconfig"
            "${pkgs.libunwind.dev}/lib/pkgconfig"
          ]
        );

        ldLibDirs = lib.concatStringsSep ":" (
          [
            "${pkgs.SDL2}/lib"
            "${pkgs.freetype}/lib"
            "${pkgs.zlib}/lib"
          ]
          ++ lib.optionals pkgs.stdenv.isLinux [
            "${pkgs.alsa-lib}/lib"
            "${pkgs.dbus}/lib"
            "${pkgs.libunwind}/lib"
          ]
        );

      in
      {
        # ── nix develop ───────────────────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Toolchains
            zig
            rustToolchain

            # C firmware build (Make + configure script)
            gnumake
            gcc
            pkg-config
            cmake
            perl     # Rockbox tools/configure is a Perl script
            python3  # Some build helpers in firmware/

            # Libraries required by the SDL build target
            SDL2 SDL2.dev
            freetype freetype.dev
            zlib zlib.dev
            libusb1 libusb1.dev

            # gRPC / protobuf (for Rust crates and webui)
            protobuf   # provides protoc
            buf

            # Dev / debugging tools
            grpcurl
            evans

            # WebUI / mobile tooling
            bun
            deno
          ] ++ linuxPkgs ++ darwinPkgs;

          shellHook = ''
            echo "Rockbox Zig development environment"
            echo "  Zig:  $(zig version)"
            echo "  Rust: $(rustc --version)"
            echo ""
            echo "SDL build (rockboxd with SDL audio):"
            echo "  cd build-lib && make lib -j\$(nproc)"
            echo "  cargo build --release -p rockbox-cli -p rockbox-server"
            echo "  cd zig && zig build"
            echo ""
            echo "Headless / cpal build (no SDL):"
            echo "  bash scripts/build-headless.sh"
            echo ""
            echo "CLI binary (rockbox):"
            echo "  cargo build --release -p rockbox"

            export PKG_CONFIG_PATH="${pkgConfigDirs}"
            export ZIG_GLOBAL_CACHE_DIR="$PWD/.zig-cache"
            export ZIG_LOCAL_CACHE_DIR="$PWD/.zig-cache"
          '' + lib.optionalString pkgs.stdenv.isLinux ''
            export LD_LIBRARY_PATH="${ldLibDirs}"
          '' + lib.optionalString pkgs.stdenv.isDarwin ''
            export DYLD_LIBRARY_PATH="${pkgs.SDL2}/lib:${pkgs.freetype}/lib:${pkgs.zlib}/lib"
            # llvm-objcopy provided by this shell (llvm 18).
            # build-headless.sh probes Homebrew paths and won't find it automatically;
            # pass it explicitly if needed:
            #   OC=$(which llvm-objcopy) bash scripts/build-headless.sh
            export ROCKBOX_LLVM_OBJCOPY="$(command -v llvm-objcopy 2>/dev/null)"
          '';
        };

        # ── nix shell (legacy alias) ──────────────────────────────────────────
        # `nix shell` is for running a package, not a dev shell.
        # This entry exposes a shell with all tools on PATH for quick one-off use.
        packages.default = pkgs.buildEnv {
          name  = "rockbox-zig-env";
          paths = with pkgs; [
            zig
            rustToolchain
            gnumake
            gcc
            pkg-config
            protobuf
            SDL2
            freetype
            zlib
            libusb1
            bun
            deno
          ] ++ linuxPkgs;
          # darwinPkgs intentionally excluded: framework paths don't compose in
          # buildEnv; they're ambient via the system SDK on macOS.
        };

        # ── Convenience build scripts (nix run .#<name>) ─────────────────────
        apps = {
          # Full headless build: firmware + Rust crates + Zig link.
          # Run from the repository root: nix run .#build-headless
          build-headless = {
            type    = "app";
            program = "${pkgs.writeShellScript "build-headless" ''
              set -euo pipefail
              exec bash scripts/build-headless.sh "$@"
            ''}";
          };

          # SDL build: make lib → cargo → zig build.
          # Run from the repository root: nix run .#build-sdl
          build-sdl = {
            type    = "app";
            program = "${pkgs.writeShellScript "build-sdl" ''
              set -euo pipefail
              NCPU=$(nproc 2>/dev/null || sysctl -n hw.logicalcpu 2>/dev/null || echo 4)
              echo "==> Step 1: firmware (build-lib)"
              (cd build-lib && make lib -j"$NCPU")
              echo "==> Step 2: Rust crates"
              cargo build --release -p rockbox-cli -p rockbox-server
              echo "==> Step 3: Zig link"
              (cd zig && zig build)
              echo "Done: zig/zig-out/bin/rockboxd"
            ''}";
          };

          # CLI binary only.
          # Run from the repository root: nix run .#build-cli
          build-cli = {
            type    = "app";
            program = "${pkgs.writeShellScript "build-cli" ''
              set -euo pipefail
              cargo build --release -p rockbox
              echo "Done: target/release/rockbox"
            ''}";
          };

          default = {
            type    = "app";
            program = "${pkgs.writeShellScript "rockboxd-info" ''
              echo "Usage:"
              echo "  nix run .#build-headless   # full headless build (rockboxd)"
              echo "  nix run .#build-sdl        # SDL build (rockboxd)"
              echo "  nix run .#build-cli        # CLI binary (rockbox)"
              echo "  nix develop                # enter dev shell"
            ''}";
          };
        };
      }
    );
}
