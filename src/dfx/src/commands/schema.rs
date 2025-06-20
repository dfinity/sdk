use crate::lib::{error::DfxResult, metadata::dfx::DfxMetadata};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use dfx_core::config::model::dfinity::{ConfigInterface, TopLevelConfigNetworks};
use dfx_core::extension::catalog::ExtensionCatalog;
use dfx_core::extension::manifest::{ExtensionDependencies, ExtensionManifest};
use schemars::schema_for;
use std::path::PathBuf;

#[derive(ValueEnum, Clone)]
enum ForFile {
    Dfx,
    Networks,
    DfxMetadata,
    ExtensionDependencies,
    ExtensionManifest,
    ExtensionCatalog,
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
        Some(ForFile::DfxMetadata) => schema_for!(DfxMetadata),
        Some(ForFile::ExtensionDependencies) => schema_for!(ExtensionDependencies),
        Some(ForFile::ExtensionManifest) => schema_for!(ExtensionManifest),
        Some(ForFile::ExtensionCatalog) => schema_for!(ExtensionCatalog),
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
