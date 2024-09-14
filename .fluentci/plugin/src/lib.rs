use anyhow::Error;
use extism_pdk::*;
use fluentci_pdk::dag;

mod helpers;

use helpers::detect_system;

#[plugin_fn]
pub fn build(args: String) -> FnResult<String> {
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
        .with_exec(vec![&format!(
            "docker buildx create --driver cloud {} || true",
            &builder
        )])?
        .with_exec(vec!["docker", "buildx", "inspect", "--bootstrap"])?
        .with_exec(vec!["docker", "buildx", "version"])?
        .with_exec(vec!["docker", "-v"])?
        .with_exec(vec!["docker", "buildx", "build", &args])?
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
