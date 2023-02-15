use std::path::PathBuf;

use crate::lib::error::DfxResult;
use dfx_core::config::model::dfinity::{ConfigInterface, TopLevelConfigNetworks};

use anyhow::Context;
use clap::{arg_enum, Parser};
use schemars::schema_for;

arg_enum! {
    enum ForFile {
        Dfx,
        Networks
    }
}

/// Prints the schema for dfx.json.
#[derive(Parser)]
pub struct SchemaOpts {
    #[clap(long, case_insensitive(true))]
    r#for: Option<ForFile>,

    /// Outputs the schema to the specified file.
    #[clap(long)]
    outfile: Option<PathBuf>,
}

pub fn exec(opts: SchemaOpts) -> DfxResult {
    let schema = match opts.r#for {
        Some(ForFile::Networks) => schema_for!(TopLevelConfigNetworks),
        _ => schema_for!(ConfigInterface),
    };
    let nice_schema =
        serde_json::to_string_pretty(&schema).context("Failed to produce pretty schema.")?;
    if let Some(outfile) = opts.outfile {
        std::fs::write(&outfile, nice_schema)
            .with_context(|| format!("Failed to write schema to {}.", outfile.to_string_lossy()))?;
    } else {
        println!("{}", nice_schema);
    }
    Ok(())
}
