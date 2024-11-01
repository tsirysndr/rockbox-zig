use std::{
    env,
    process::{Command, Stdio},
};

use anyhow::Error;

pub mod api {
    #[path = ""]
    pub mod rockbox {

        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;
    }
}

pub fn install_rockboxd() -> Result<(), Error> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("type rockboxd > /dev/null 2>&1 || curl -fsSL https://raw.githubusercontent.com/tsirysndr/rockbox-zig/HEAD/install.sh | bash")
        .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
        .spawn()?;
    child.wait()?;
    Ok(())
}

pub fn wait_for_rockboxd(port: u32, timeout: Option<u32>) -> Result<(), Error> {
    setup_pkgx()?;
    let cmd = format!(
        "pkgx deno run -A npm:wait-port localhost:{} -t {}",
        port,
        timeout.unwrap_or(60) * 1000
    );
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let status = child.wait()?;

    if !status.success() {
        return Err(Error::msg("Timeout waiting for Rockbox server"));
    }

    Ok(())
}

pub fn setup_pkgx() -> Result<(), Error> {
    let path = format!(
        "{}:{}",
        env::var("PATH")?,
        format!("{}/.local/bin", env::var("HOME")?)
    );
    env::set_var("PATH", &path);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("type pkgx > /dev/null 2>&1 || curl -fsS https://pkgx.sh | sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    child.wait()?;

    Ok(())
}
