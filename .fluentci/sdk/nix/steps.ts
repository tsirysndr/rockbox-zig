import { Container } from "../client.gen.ts";

export const withNix = (ctr: Container) =>
  ctr
    .withExec([
      "sh",
      "-c",
      "[ -f /etc/nix/group ] && cp /etc/nix/group /etc/group; exit 0",
    ])
    .withExec([
      "sh",
      "-c",
      "[ -f /etc/nix/passwd ] && cp /etc/nix/passwd /etc/passwd; exit 0",
    ])
    .withExec([
      "sh",
      "-c",
      "[ -f /etc/nix/shadow ] && cp /etc/nix/shadow /etc/shadow; exit 0",
    ])
    .withExec([
      "sh",
      "-c",
      '[ ! -f "/nix/receipt.json" ] && curl --proto =https --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install linux --extra-conf "sandbox = false" --init none --no-confirm; exit 0',
    ])
    .withExec(["cp", "/etc/group", "/etc/nix/group"])
    .withExec(["cp", "/etc/passwd", "/etc/nix/passwd"])
    .withExec(["cp", "/etc/shadow", "/etc/nix/shadow"])
    .withExec([
      "sed",
      "-i",
      "s/auto-allocate-uids = true/auto-allocate-uids = false/g",
      "/etc/nix/nix.conf",
    ])
    .withEnvVariable("PATH", "${PATH}:/nix/var/nix/profiles/default/bin", {
      expand: true,
    })
    .withExec(["nix", "run", "nixpkgs#hello"]);

export const withDevbox = (ctr: Container) =>
  withNix(ctr)
    .withExec(["adduser", "--disabled-password", "devbox"])
    .withExec(["addgroup", "devbox", "nixbld"])
    .withEnvVariable("FORCE", "1")
    .withExec(["sh", "-c", "curl -fsSL https://get.jetpack.io/devbox | bash"])
    .withExec(["devbox", "version"]);

export const withPackageFromDevbox = (ctr: Container, pkgs: string[]) =>
  withDevbox(ctr).withExec(["devbox", "global", "add", ...pkgs]);

export const withDevboxExec = (ctr: Container, cmds: string[]) =>
  cmds.reduce(
    (ctr, cmd) =>
      ctr.withExec(["sh", "-c", `eval "$(devbox global shellenv)" && ${cmd}`]),
    ctr
  );

export const withDevenv = (ctr: Container) =>
  withNix(ctr)
    .withExec(["adduser", "--disabled-password", "devenv"])
    .withExec(["addgroup", "devenv", "nixbld"])
    .withEnvVariable("USER", "root")
    .withExec([
      "sh",
      "-c",
      'echo "trusted-users = root $USER" | tee -a /etc/nix/nix.conf',
    ])
    .withExec([
      "nix",
      "profile",
      "install",
      "--accept-flake-config",
      "github:cachix/cachix",
    ])
    .withExec(["cachix", "use", "devenv"])
    .withExec([
      "nix",
      "profile",
      "install",
      "--accept-flake-config",
      "github:cachix/devenv/latest",
    ])
    .withExec(["devenv", "version"]);

export const withFlox = (ctr: Container) =>
  withNix(ctr)
    .withExec(["adduser", "--disabled-password", "flox"])
    .withExec(["addgroup", "flox", "nixbld"])
    .withExec([
      "sh",
      "-c",
      "echo 'extra-trusted-substituters = https://cache.floxdev.com' | tee -a /etc/nix/nix.conf && echo 'extra-trusted-public-keys = flox-store-public-0:8c/B+kjIaQ+BloCmNkRUKwaVPFWkriSAd0JJvuDu4F0=' | tee -a /etc/nix/nix.conf",
    ])
    .withExec([
      "nix",
      "profile",
      "install",
      "--impure",
      "--experimental-features",
      "nix-command flakes auto-allocate-uids",
      "--accept-flake-config",
      "github:flox/floxpkgs#flox.fromCatalog",
    ])
    .withExec(["flox", "--version"]);
