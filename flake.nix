{
  description = "Rockbox Zig project flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.stdenv.mkDerivation rec {
          pname = "rockbox-zig";
          version = "0.1.0";

          src = ./.;

          nativeBuildInputs = with pkgs; [
            zig
            pkg-config
            gnumake
          ];

          buildInputs = with pkgs; [
            SDL2
            freetype
            zlib
            libunwind
          ];

          # For Rust dependencies if needed
          # nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
          #   cargo
          #   rustc
          # ]);

          buildPhase = ''
            runHook preBuild

            # Set up cache directory
            export ZIG_GLOBAL_CACHE_DIR=$TMPDIR/zig-cache
            export ZIG_LOCAL_CACHE_DIR=$TMPDIR/zig-local-cache

            # Build the project
            zig build --prefix $out --cache-dir $ZIG_LOCAL_CACHE_DIR

            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            # zig build already installs to $out
            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Rockbox Zig project";
            license = licenses.mit; # adjust as needed
            platforms = platforms.linux;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Zig toolchain
            zig

            # Build tools
            gnumake
            pkg-config
            gcc

            # Libraries
            SDL2
            freetype
            zlib
            libunwind

            # Protocol buffers
            protobuf
            buf

            # JavaScript runtimes
            deno
            bun

            # Rust toolchain (if needed)
            cargo
            rustc

            # Development tools
            evans
            grpcurl
          ];

          # Set up environment variables
          shellHook = ''
            echo "🚀 Rockbox Zig development environment"
            echo "Zig version: $(zig version)"

            # Set up Zig cache directories
            export ZIG_GLOBAL_CACHE_DIR="$PWD/.zig-cache"
            export ZIG_LOCAL_CACHE_DIR="$PWD/.zig-cache"

            # Ensure cache directories exist
            mkdir -p "$ZIG_GLOBAL_CACHE_DIR"
            mkdir -p "$ZIG_LOCAL_CACHE_DIR"

            echo "Build with: zig build"
            echo "Run with: zig build run"
            echo "Test with: zig build test"
          '';

          # Environment variables for libraries
          PKG_CONFIG_PATH = "${pkgs.SDL2.dev}/lib/pkgconfig:${pkgs.freetype.dev}/lib/pkgconfig:${pkgs.zlib.dev}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.SDL2}/lib:${pkgs.freetype}/lib:${pkgs.zlib}/lib:${pkgs.libunwind}/lib";
        };

        # Convenience apps
        apps = {
          default = flake-utils.lib.mkApp {
            drv = self.packages.${system}.default;
          };

          build = {
            type = "app";
            program = "${pkgs.writeShellScript "zig-build" ''
              ${pkgs.zig}/bin/zig build "$@"
            ''}";
          };

          run = {
            type = "app";
            program = "${pkgs.writeShellScript "zig-run" ''
              ${pkgs.zig}/bin/zig build run -- "$@"
            ''}";
          };

          test = {
            type = "app";
            program = "${pkgs.writeShellScript "zig-test" ''
              ${pkgs.zig}/bin/zig build test -- "$@"
            ''}";
          };
        };
      });
}