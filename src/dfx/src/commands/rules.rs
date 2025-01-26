use std::fs::File;
use std::io::Write;

use crate::config::cache::DiskBasedCache;
use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use clap::Parser;
use tokio::runtime::Runtime;

/// Output dependencies in Make format
#[derive(Parser)]
pub struct RulesOpts {
    /// File to output make rules
    #[arg(long, short, value_name = "FILE")]
    output: Option<String>,
}

pub fn exec(env1: &dyn Environment, opts: RulesOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env1, None)?;
    let log = env.get_logger();

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    let mut output_file: Box<dyn Write> = match opts.output {
        Some(filename) => Box::new(File::open(filename)?),
        None => Box::new(std::io::stdout()),
    };

    match &config.get_config().canisters {
        Some(canisters) => {
            output_file.write_fmt(format_args!(".PHONY:"))?;
            for canister in canisters {
                output_file.write_fmt(format_args!(" canister:{}", canister.0))?;
            };
            output_file.write_fmt(format_args!("\n\n"))?;
        }
        None => {}
    };

    Ok(())
}
