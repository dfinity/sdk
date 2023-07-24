use crate::lib::error::DfxResult;
use anyhow::Context;
use clap::{Parser, ValueEnum};
use dfx_core::config::model::dfinity::{ConfigInterface, TopLevelConfigNetworks};
use schemars::schema_for;
use std::path::PathBuf;

#[derive(ValueEnum, Clone)]
enum ForFile {
    Dfx,
    Networks,
}

/// Prints the schema for dfx.json.
#[derive(Parser)]
pub struct SchemaOpts {
    #[arg(long, value_enum)]
    r#for: Option<ForFile>,

    /// Outputs the schema to the specified file.
    #[arg(long)]
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
