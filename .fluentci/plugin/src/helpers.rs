use anyhow::Error;
use fluentci_pdk::dag;

pub fn detect_system() -> Result<(String, String), Error> {
    let os = match dag().get_os()?.as_str() {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        _ => return Err(Error::msg("unsupported os")),
    };
    let arch = match dag().get_arch()?.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "arm" => "arm-v7",
        _ => return Err(Error::msg("unsupported arch")),
    };

    Ok((os.into(), arch.into()))
}

pub fn download_release(version: String) -> Result<String, Error> {
    dag()
        .pipeline("mkdir")?
        .pkgx()?
        .with_exec(vec!["rm", "-rf", "/tmp/rbrelease"])?
        .with_exec(vec!["mkdir", "/tmp/rbrelease"])?
        .stdout()?;

    let stdout = dag()
        .pipeline("download-release")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease")?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockbox_{}_x86_64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockboxd_{}_x86_64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockbox-codecs-{}-x86_64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockbox-assets-{}-x86_64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockbox_{}_aarch64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockboxd_{}_aarch64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockbox-codecs-{}-aarch64-linux.tar.gz", version, version)])?
        .with_exec(vec!["pkgx", "wget@1.21.4", &format!("https://github.com/tsirysndr/rockbox-zig/releases/download/{}/rockbox-assets-{}-aarch64-linux.tar.gz", version, version)])?
        .with_exec(vec!["mkdir", "-p", "amd64/bin", "amd64/lib/rockbox", "amd64/share/rockbox", "arm64/bin", "arm64/lib/rockbox", "arm64/share/rockbox"])?
        .stdout()?;

    dag()
        .pipeline("extract-bin-amd64")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease/amd64/bin")?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!("/tmp/rbrelease/rockbox_{}_x86_64-linux.tar.gz", version),
        ])?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!("/tmp/rbrelease/rockboxd_{}_x86_64-linux.tar.gz", version),
        ])?
        .stdout()?;

    dag()
        .pipeline("extract-lib-amd64")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease/amd64/lib/rockbox")?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!(
                "/tmp/rbrelease/rockbox-codecs-{}-x86_64-linux.tar.gz",
                version
            ),
        ])?
        .stdout()?;

    dag()
        .pipeline("extract-share-amd64")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease/amd64/share/rockbox")?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!(
                "/tmp/rbrelease/rockbox-assets-{}-x86_64-linux.tar.gz",
                version
            ),
        ])?
        .stdout()?;

    dag()
        .pipeline("extract-bin-arm64")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease/arm64/bin")?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!("/tmp/rbrelease/rockbox_{}_aarch64-linux.tar.gz", version),
        ])?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!("/tmp/rbrelease/rockboxd_{}_aarch64-linux.tar.gz", version),
        ])?
        .stdout()?;

    dag()
        .pipeline("extract-lib-arm64")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease/arm64/lib/rockbox")?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!(
                "/tmp/rbrelease/rockbox-codecs-{}-aarch64-linux.tar.gz",
                version
            ),
        ])?
        .stdout()?;

    dag()
        .pipeline("extract-share-arm64")?
        .pkgx()?
        .with_workdir("/tmp/rbrelease/arm64/share/rockbox")?
        .with_exec(vec![
            "tar",
            "xzvf",
            &format!(
                "/tmp/rbrelease/rockbox-assets-{}-aarch64-linux.tar.gz",
                version
            ),
        ])?
        .stdout()?;

    Ok(stdout)
}
