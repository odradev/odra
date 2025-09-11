use quote::ToTokens;
use std::{fs::File, io::Write, path::Path, process::Command};

use crate::error::{Error, Result};

pub fn fmt<P: AsRef<Path>>(path: &P) -> Result<()> {
    Command::new("cargo")
        .arg("fmt")
        .current_dir(path)
        .status()?;
    Ok(())
}

pub fn build<P: AsRef<Path>>(path: &P) -> Result<()> {
    Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("pkg-web")
        .arg("--release")
        .arg(
            path.as_ref()
                .to_str()
                .ok_or(Error::InvalidSchemaPath)?
        )
        .status()?;
    Ok(())
}

pub fn write<P: AsRef<Path>, C: ToTokens>(path: &P, code: C) -> Result<()> {
    let path = path.as_ref().join("src/lib.rs");
    let mut file = File::create(path)?;
    file.write_all(code.to_token_stream().to_string().as_bytes())?;
    Ok(())
}
