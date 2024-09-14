use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn hello(args: String) -> FnResult<String> {
    let stdout = dag()
        .pipeline("hello")?
        .with_exec(vec!["echo Hello", &args])?
        .stdout()?;
    Ok(stdout)
}
