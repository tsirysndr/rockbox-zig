use anyhow::Error;

pub fn install_dependencies() -> Result<(), Error> {
    rockbox_typesense::setup()?;
    Ok(())
}
