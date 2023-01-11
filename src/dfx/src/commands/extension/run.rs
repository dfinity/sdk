use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::{anyhow};
use clap::Parser;

use std::ffi::OsString;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specifies the name of the extension to run.
    extension_name: OsString,
    /// Specifies the parameters to pass to the extension.
    params: Vec<OsString>,
}

impl From<Vec<OsString>> for RunOpts {
    fn from(params: Vec<OsString>) -> Self {
        let (extension_name, params) = params.split_first().unwrap();
        RunOpts {
            extension_name: extension_name.clone(),
            params: params.to_vec(),
        }
    }
}

pub fn exec(env: &dyn Environment, opts: RunOpts) -> DfxResult<()> {
    println!("params: {:?}", opts);
    if let Ok(mut extension_binary) = env
        .get_cache()
        .get_extension_binary(opts.extension_name.to_str().unwrap())
    {
        return extension_binary
            .args(&opts.params)
            .spawn()
            .expect("failed to execute process")
            .wait()
            .expect("failed to wait on child")
            .code()
            .map_or(Ok(()), |code| {
                Err(anyhow!("Extension exited with code {}", code))
            });
    } else {
        println!("extension {:?} does cannot be found", opts.extension_name);
    }

    Ok(())
}
