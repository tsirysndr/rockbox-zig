{
  description = "Rockbox Zig project flake";

  inputs = {
     nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    zig2nix.url = "github:Cloudef/zig2nix";
  };

  outputs = { zig2nix, nixpkgs, ... }: let
    flake-utils = zig2nix.inputs.flake-utils;
  in (flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
          inherit system;
        };

      inherit (pkgs) lib;
      # Zig flake helper
      # Check the flake.nix in zig2nix project for more options:
      # <https://github.com/Cloudef/zig2nix/blob/master/flake.nix>
      env = zig2nix.outputs.zig-env.${system} { zig = pkgs.zig; };
      system-triple = env.lib.zigTripleFromString system;
    in with builtins; with env.lib; with env.pkgs.lib; rec {
      # nix build .#target.{zig-target}
      # e.g. nix build .#target.x86_64-linux-gnu
      packages.target = genAttrs allTargetTriples (target: env.packageForTarget target ({
        src = cleanSource ./.;

        nativeBuildInputs = with env.pkgs; [];
        buildInputs = with env.pkgsForTarget target; [
          pkgs.zig
          pkgs.SDL_compat
          pkgs.freetype
          pkgs.gnumake
          pkgs.pkg-config
          pkgs.gcc
          pkgs.zlib
          pkgs.cargo
          pkgs.rustc
          pkgs.protobuf
          pkgs.buf
          pkgs.libunwind
        ];

        # Smaller binaries and avoids shipping glibc.
        zigPreferMusl = false;

        # This disables LD_LIBRARY_PATH mangling, binary patching etc...
        # The package won't be usable inside nix.
        zigDisableWrap = false;
      } // optionalAttrs (!pathExists ./build.zig.zon) {
        pname = "rockbox-zig";
        version = "0.1.0";
      }));

      # nix build .
      packages.default = packages.target.${system-triple}.override {
        # Prefer nix friendly settings.
        zigPreferMusl = false;
        zigDisableWrap = false;
      };

      # For bundling with nix bundle for running outside of nix
      # example: https://github.com/ralismark/nix-appimage
      apps.bundle.target = genAttrs allTargetTriples (target: let
        pkg = packages.target.${target};
      in {
        type = "app";
        program = "${pkg}/bin/default";
      });

      # default bundle
      apps.bundle.default = apps.bundle.target.${system-triple};

      # nix run .
      apps.default = env.app [] "zig build run -- \"$@\"";

      # nix run .#build
      apps.build = env.app [] "zig build \"$@\"";

      # nix run .#test
      apps.test = env.app [] "zig build test -- \"$@\"";

      # nix run .#docs
      apps.docs = env.app [] "zig build docs -- \"$@\"";

      # nix run .#deps
      apps.deps = env.showExternalDeps;

      # nix run .#zon2json
      apps.zon2json = env.app [env.zon2json] "zon2json \"$@\"";

      # nix run .#zon2json-lock
      apps.zon2json-lock = env.app [env.zon2json-lock] "zon2json-lock \"$@\"";

      # nix run .#zon2nix
      apps.zon2nix = env.app [env.zon2nix] "zon2nix \"$@\"";

      # nix develop
      devShells.default = env.mkShell {
        nativeBuildInputs = with env.pkgs; [
          pkgs.zig
          pkgs.SDL_compat
          pkgs.freetype
          pkgs.gnumake
          pkgs.pkg-config
          pkgs.gcc
          pkgs.zlib
          pkgs.cargo
          pkgs.rustc
          pkgs.protobuf
          pkgs.buf
          pkgs.libunwind
          pkgs.evans
          pkgs.grpcurl
        ];
      };
    }));
}