{
  description = "A Nix-flake-based Zig development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
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