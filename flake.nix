{
  description = "A Nix-flake-based Zig development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    zig2nix.url = "github:Cloudef/zig2nix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    zig2nix,
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
      env = zig2nix.outputs.zig-env.${system} { zig = pkgs.zig; };
      system-triple = env.lib.zigTripleFromString system;
    in with builtins; with env.lib; with env.pkgs.lib; rec {
        packages.target = genAttrs allTargetTriples (target: env.packageForTarget target ({
        src = cleanSource ./.;

        nativeBuildInputs = with env.pkgs; [];
        buildInputs = with env.pkgsForTarget target; [];

        # Smaller binaries and avoids shipping glibc.
        # zigPreferMusl = true;

        # This disables LD_LIBRARY_PATH mangling, binary patching etc...
        # The package won't be usable inside nix.
        zigDisableWrap = true;
      } // optionalAttrs (!pathExists ./build.zig.zon) {
        pname = "rockbox-zig";
        version = "0.0.0";
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
        program = "${pkg}/bin/master";
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
      
      devShells.default = pkgs.mkShell {
         buildInputs = [
          pkgs.zig
         ];
         shellHook = ''
           readonly YELLOW="$(tput setaf 3 2>/dev/null)"
           readonly NO_COLOR="$(tput sgr0 2>/dev/null)"
           echo $YELLOW
           cat <<EOF
               __________               __   ___.
     Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
     Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
     Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
     Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \\
                       \/            \/     \/    \/            \/
EOF
           echo $NO_COLOR
           echo 'Zig' $(which zig)
           echo 'Welcome to Rockbox development shell!'
          '';
      };
    });
}
