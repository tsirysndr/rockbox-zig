import { Directory, Container, Platform, dag, env } from "../deps.ts";
import { getDirectory } from "./helpers.ts";

export enum Job {
  build = "build",
  publish = "publish",
}

export const exclude = [];

const platforms: Platform[] = [
  "linux/amd64" as Platform,
  "linux/arm64" as Platform,
];

/**
 * @function
 * @description Build Rockbox binary
 * @param src {src: string | Directory | undefined}
 * @returns {string}
 */
export async function build(
  src: string | Directory | undefined = ".",
  platform: string = "linux/amd64"
): Promise<Container> {
  const ZIG_VERSION = env.get("ZIG_VERSION") || "0.13.0";
  const RUST_VERSION = env.get("RUST_VERSION") || "1.81-bookworm";
  const context = await getDirectory(src);
  const cacheSuffix = platform.replace("/", "-");
  const ctr = dag
    .container({
      platform: platform as Platform,
    })
    .from(`rust:${RUST_VERSION}`)
    .withExec(["apt-get", "update"])
    .withExec([
      "apt-get",
      "install",
      "-y",
      "build-essential",
      "libusb-dev",
      "libsdl1.2-dev",
      "libfreetype6-dev",
      "libunwind-dev",
      "curl",
      "zip",
      "unzip",
      "protobuf-compiler",
    ])
    .withMountedCache(
      "/root/.local",
      dag.cacheVolume(`rockbox-local-${cacheSuffix}`)
    )
    .withExec(["rm", "-rf", "/root/.local/share/pkgx"])
    .withExec(["sh", "-c", "curl -Ssf https://pkgx.sh | sh"])
    .withExec(["pkgx", "install", `zig@${ZIG_VERSION}`])
    .withDirectory("/app", context)
    .withWorkdir("/app")
    .withExec(["mkdir", "-p", "build", "/root/.local/lib/rockbox"])
    .withMountedCache(
      "/app/build",
      dag.cacheVolume(`rockbox-build-${cacheSuffix}`)
    )
    .withMountedCache(
      "/app/.zig-cache",
      dag.cacheVolume(`zig-cache-${cacheSuffix}`)
    )
    .withMountedCache("/app/zig-out", dag.cacheVolume(`zig-out-${cacheSuffix}`))
    .withMountedCache(
      "/app/target",
      dag.cacheVolume(`rust-cache-${cacheSuffix}`)
    )
    .withWorkdir("/app/build")
    .withExec([
      "bash",
      "-c",
      "../tools/configure --target=sdlapp --type=N --lcdwidth=320 --lcdheight=240 --prefix=$HOME/.local",
    ])
    .withExec(["bash", "-c", "make ziginstall -j$(nproc)"]);

  await ctr.stdout();
  return ctr;
}

/**
 * @function
 * @description Publish Rockbox Docker image
 * @param src {src: string | Directory | undefined}
 * @returns {string}
 */
export async function publish(
  src: string | Directory | undefined = "."
): Promise<string> {
  const DEBIAN_VERSION = env.get("DEBIAN_VERSION") || "bookworm";
  const platformVariants: Array<Container> = [];

  for (const platform of platforms) {
    await build(src, platform as string);
  }

  for (const platform of platforms) {
    const cacheSuffix = platform.replace("/", "-");
    const ctr = dag
      .container({
        platform: platform as Platform,
      })
      .from(`debian:${DEBIAN_VERSION}`)
      .withExec(["apt-get", "update"])
      .withExec([
        "apt-get",
        "install",
        "-y",
        "libusb-dev",
        "libsdl1.2-dev",
        "libunwind-dev",
      ])
      .withMountedCache(
        "/cache",
        dag.cacheVolume(`rockbox-local-${cacheSuffix}`)
      )
      .withExec([
        "bash",
        "-c",
        "mkdir -p /root/.local && cp -r /cache/* /root/.local && cp /cache/bin/rockbox /usr/bin",
      ])
      .withExec([
        "bash",
        "-c",
        "ls -la /root/.local/bin /root/.local/lib/rockbox/* /root/.local/share/rockbox",
      ])
      .withEnvVariable("SDL_VIDEODRIVER", "dummy")
      .withEntrypoint(["rockbox"]);

    await ctr.stdout();
    platformVariants.push(ctr);
  }

  const ROCKBOX_IMAGE =
    env.get("ROCKBOX_IMAGE") || "ghcr.io/tsirysndr/rockbox:latest";

  const imageDigest = await dag
    .container()
    .withRegistryAuth(
      ROCKBOX_IMAGE,
      env.get("GITHUB_USERNAME")!,
      dag.setSecret("GITHUB_TOKEN", env.get("GITHUB_TOKEN")!)
    )
    .publish(ROCKBOX_IMAGE, { platformVariants });

  console.log(imageDigest);

  return "Successfully published Rockbox Docker image";
}

export type JobExec =
  | ((src?: string, platform?: string) => Promise<Container>)
  | ((
      src?: string,
      options?: {
        ignore: string[];
      }
    ) => Promise<string>);

export const runnableJobs: Record<Job, JobExec> = {
  [Job.build]: build,
  [Job.publish]: publish,
};

export const jobDescriptions: Record<Job, string> = {
  [Job.build]: "Build Rockbox binary",
  [Job.publish]: "Publish Rockbox Docker image",
};
