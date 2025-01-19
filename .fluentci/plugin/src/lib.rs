use anyhow::Error;
use extism_pdk::*;
use fluentci_pdk::dag;

mod helpers;

use helpers::{detect_system, download_release};

#[plugin_fn]
pub fn build(_arg: String) -> FnResult<String> {
    let stdout = dag()
        .pipeline("build")?
        .pkgx()?
        .with_workdir("./webui/rockbox")?
        .with_exec(vec!["sudo", "apt-get", "update"])?
        .with_exec(vec![
            "sudo",
            "apt-get install",
            "-y",
            "build-essential",
            "libusb-dev",
            "libsdl2-dev",
            "libfreetype6-dev",
            "libunwind-dev",
            "curl",
            "wget",
            "zip",
            "unzip",
            "cmake",
        ])?
        .with_exec(vec![
            "pkgm",
            "install",
            "zig@0.13.0",
            "buf",
            "deno",
            "bun",
            "node@18",
        ])?
        .with_exec(vec!["deno install"])?
        .with_exec(vec!["deno", "run", "build"])?
        .stdout()?;

    // download & install protoc
    dag()
        .pipeline("protoc")?
        .pkgx()?
        .with_exec(vec![
            "git",
            "clone",
            "https://github.com/protocolbuffers/protobuf",
        ])?
        .with_exec(vec![
            "cd protobuf &&",
            "git checkout v29.2 &&",
            "git submodule update --init --recursive &&",
            "mkdir build &&",
            "cd build &&",
            "cmake .. && sudo make install -j$(nproc)",
        ])?
        .stdout()?;

    dag()
        .pipeline("mkdir")?
        .pkgx()?
        .with_exec(vec!["mkdir", "-p", "build"])?
        .stdout()?;

    dag()
        .pipeline("build")?
        .pkgx()?
        .with_workdir("build")?
        .with_exec(vec![
            "../tools/configure",
            "--target=sdlapp",
            "--type=N",
            "--lcdwidth=320",
            "--lcdheight=240",
            "--prefix=/usr/local",
        ])?
        .with_exec(vec!["sudo", "make", "ziginstall", "-j$(nproc)"])?
        .stdout()?;

    Ok(stdout)
}

#[plugin_fn]
pub fn deb(version: String) -> FnResult<String> {
    download_release(version.clone())?;
    dag()
        .pipeline("copy-src-deb")?
        .pkgx()?
        .with_exec(vec!["mkdir", "-p", "dist/debian/amd64/usr/local"])?
        .with_exec(vec![
            "cp",
            "-r",
            "/tmp/rbrelease/amd64/*",
            "dist/debian/amd64/usr/local",
        ])?
        .with_exec(vec!["mkdir", "-p", "dist/debian/arm64/usr/local"])?
        .with_exec(vec![
            "cp",
            "-r",
            "/tmp/rbrelease/arm64/*",
            "dist/debian/arm64/usr/local",
        ])?
        .stdout()?;

    let stdout = dag()
        .pipeline("deb")?
        .pkgx()?
        .with_workdir("dist/debian")?
        .with_exec(vec!["dpkg-deb", "--build", "arm64"])?
        .with_exec(vec!["dpkg-deb", "--build", "amd64"])?
        .with_exec(vec![
            "mv",
            "arm64.deb",
            &format!("rockbox_{}_arm64.deb", version),
        ])?
        .with_exec(vec![
            "mv",
            "amd64.deb",
            &format!("rockbox_{}_amd64.deb", version),
        ])?
        .stdout()?;

    Ok(stdout)
}

#[plugin_fn]
pub fn rpm(version: String) -> FnResult<String> {
    download_release(version.clone())?;
    dag()
        .pipeline("copy-src-rpm")?
        .pkgx()?
        .with_exec(vec!["mkdir", "-p", "~/rpmbuild/SOURCES/amd64/usr/local"])?
        .with_exec(vec!["mkdir", "-p", "~/rpmbuild/SOURCES/arm64/usr/local"])?
        .with_exec(vec![
            "cp",
            "-r",
            "/tmp/rbrelease/amd64/*",
            "~/rpmbuild/SOURCES/amd64/usr/local",
        ])?
        .with_exec(vec![
            "cp",
            "-r",
            "/tmp/rbrelease/arm64/*",
            "~/rpmbuild/SOURCES/arm64/usr/local",
        ])?
        .stdout()?;

    let stdout = dag()
        .pipeline("rpm")?
        .pkgx()?
        .with_workdir("dist/rpm")?
        .with_exec(vec!["rpmbuild", "--bb", "amd64/rockbox.spec"])?
        .stdout()?;

    Ok(stdout)
}

#[plugin_fn]
pub fn release(_args: String) -> FnResult<String> {
    let tag = dag().get_env("TAG")?;
    let gh_token = dag().get_env("GH_TOKEN")?;

    if tag.is_empty() || gh_token.is_empty() {
        return Ok("TAG, GH_TOKEN not set, skipping release".into());
    }

    let os = dag().get_os()?;
    let arch = dag().get_arch()?;
    let target = format!("{}-{}", arch, os);

    if os != "linux" {
        return Ok("Only linux is supported for release".into());
    }

    if arch != "x86_64" && arch != "aarch64" {
        return Ok("Only x86_64 and aarch64 are supported for release".into());
    }

    dag().set_envs(vec![("TARGET".into(), target)])?;

    dag()
        .pipeline("archive")?
        .pkgx()?
        .with_exec(vec![
            "cd target/release && tar czvf rockbox_${TAG}_${TARGET}.tar.gz rockbox",
        ])?
        .with_exec(vec![
            "cd target/release && sha256sum rockbox_${TAG}_${TARGET}.tar.gz > rockbox_${TAG}_${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "cd zig-out/bin && tar czvf rockboxd_${TAG}_${TARGET}.tar.gz rockboxd",
        ])?
        .with_exec(vec![
            "cd zig-out/bin && sha256sum rockboxd_${TAG}_${TARGET}.tar.gz > rockboxd_${TAG}_${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "cd /usr/local/lib/rockbox && tar czvf rockbox-codecs-${TAG}-${TARGET}.tar.gz *",
        ])?
        .with_exec(vec![
            "cd /usr/local/lib/rockbox && sha256sum rockbox-codecs-${TAG}-${TARGET}.tar.gz > rockbox-codecs-${TAG}-${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "cd /usr/local/share/rockbox && tar czvf rockbox-assets-${TAG}-${TARGET}.tar.gz *",
        ])?
        .with_exec(vec![
            "cd /usr/local/share/rockbox && sha256sum rockbox-assets-${TAG}-${TARGET}.tar.gz > rockbox-assets-${TAG}-${TARGET}.tar.gz.sha256",
        ])?
        .stdout()?;

    let stdout = dag()
        .pipeline("release")?
        .pkgx()?
        .with_packages(vec!["gh"])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "target/release/rockbox_${TAG}_${TARGET}.tar.gz",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "target/release/rockbox_${TAG}_${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "zig-out/bin/rockboxd_${TAG}_${TARGET}.tar.gz",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "zig-out/bin/rockboxd_${TAG}_${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "/usr/local/lib/rockbox/rockbox-codecs-${TAG}-${TARGET}.tar.gz",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "$TAG",
            "/usr/local/lib/rockbox/rockbox-codecs-${TAG}-${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "/usr/local/share/rockbox/rockbox-assets-${TAG}-${TARGET}.tar.gz",
        ])?
        .with_exec(vec![
            "gh",
            "release",
            "upload",
            "${TAG}",
            "/usr/local/share/rockbox/rockbox-assets-${TAG}-${TARGET}.tar.gz.sha256",
        ])?
        .with_exec(vec![
            "rm",
            "/usr/local/lib/rockbox/*.tar.gz*",
            "/usr/local/share/rockbox/*.tar.gz*",
        ])?
        .stdout()?;

    Ok(stdout)
}

#[plugin_fn]
pub fn build_docker(args: String) -> FnResult<String> {
    let version = dag()
        .get_env("BUILDX_VERSION")
        .unwrap_or("v0.17.1-desktop.1".into());
    let version = match version.as_str() {
        "" => "v0.17.1-desktop.1".into(),
        _ => version,
    };
    let (os, arch) = detect_system()?;

    let buildx_download_url = format!(
        "https://github.com/docker/buildx-desktop/releases/download/{}/buildx-{}.{}-{}",
        version, version, os, arch
    );

    let buildx_plugin = format!("buildx-{}.{}-{}", version, os, arch);
    let stdout = dag()
        .pipeline("build")?
        .pkgx()?
        .with_exec(vec!["pkgx", "install", "docker", "wget"])?
        .with_exec(vec![&format!(
            r#"
          if [ ! -f $HOME/.docker/cli-plugins/docker-buildx ]; then
            wget {};
            chmod +x {};
            mkdir -p $HOME/.docker/cli-plugins;
            mv {} $HOME/.docker/cli-plugins/docker-buildx;
          fi
        "#,
            buildx_download_url, buildx_plugin, buildx_plugin
        )])?
        .with_exec(vec!["docker buildx rm builder || true"])?
        .with_exec(vec![
            "docker", "buildx", "create", "--name", "builder", "--use",
        ])?
        .with_exec(vec!["docker", "buildx", "inspect", "--bootstrap"])?
        .with_exec(vec!["docker", "buildx", "version"])?
        .with_exec(vec!["docker", "-v"])?
        .with_exec(vec!["docker", "buildx", "build", &args])?
        .stdout()?;
    Ok(stdout)
}

#[plugin_fn]
pub fn build_cloud(args: String) -> FnResult<String> {
    let builder = dag().get_env("BUILDX_BUILDER")?;
    if builder.is_empty() {
        return Err(Error::msg("BUILDX_BUILDER must be set").into());
    }

    let version = dag()
        .get_env("BUILDX_VERSION")
        .unwrap_or("v0.17.1-desktop.1".into());
    let version = match version.as_str() {
        "" => "v0.17.1-desktop.1".into(),
        _ => version,
    };
    let (os, arch) = detect_system()?;

    let buildx_download_url = format!(
        "https://github.com/docker/buildx-desktop/releases/download/{}/buildx-{}.{}-{}",
        version, version, os, arch
    );

    let buildx_plugin = format!("buildx-{}.{}-{}", version, os, arch);
    let builder_name = format!("cloud-{}", builder.replace("/", "-"));
    let stdout = dag()
        .pipeline("build")?
        .pkgx()?
        .with_exec(vec!["pkgx", "install", "docker", "wget"])?
        .with_exec(vec![&format!(
            r#"
          if [ ! -f $HOME/.docker/cli-plugins/docker-buildx ]; then
            wget {};
            chmod +x {};
            mkdir -p $HOME/.docker/cli-plugins;
            mv {} $HOME/.docker/cli-plugins/docker-buildx;
          fi
        "#,
            buildx_download_url, buildx_plugin, buildx_plugin
        )])?
        .with_exec(vec!["docker buildx rm builder || true"])?
        .with_exec(vec!["docker", "buildx", "version"])?
        .with_exec(vec!["docker", "-v"])?
        .with_exec(vec![&format!(
            "docker buildx create --driver cloud {} || true",
            &builder
        )])?
        .with_exec(vec![
            "docker",
            "buildx",
            "build",
            "--builder",
            &builder_name,
            &args,
        ])?
        .stdout()?;
    Ok(stdout)
}

#[plugin_fn]
pub fn publish(args: String) -> FnResult<String> {
    let registry_password = dag().get_env("REGISTRY_PASSWORD")?;
    let registry_url = dag().get_env("REGISTRY_URL")?;
    let registry_user = dag().get_env("REGISTRY_USER")?;

    if registry_password.is_empty() || registry_url.is_empty() || registry_user.is_empty() {
        return Err(
            Error::msg("REGISTRY_PASSWORD, REGISTRY_URL, REGISTRY_USER must be set").into(),
        );
    }

    let stdout = dag()
        .pipeline("publish")?
        .pkgx()?
        .with_exec(vec!["pkgx", "install", "docker"])?
        .with_exec(vec!["echo $REGISTRY_PASSWORD | docker login $REGISTRY_URL -u $REGISTRY_USER --password-stdin"])?
        .with_exec(vec!["docker", "push", &args])?
        .stdout()?;
    Ok(stdout)
}
