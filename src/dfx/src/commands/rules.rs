use std::fs::File;
use std::io::Write;

use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::builders::CanisterBuilder;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::builders::custom::CustomBuilder;
use clap::Parser;

/// Output dependencies in Make format
#[derive(Parser)]
pub struct RulesOpts {
    /// File to output make rules
    #[arg(long, short, value_name = "FILE")]
    output: Option<String>,
}

pub fn exec(env1: &dyn Environment, opts: RulesOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env1, None)?;
    // let log = env.get_logger();

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

    let builder = CustomBuilder::new(env1)?; // TODO: hack
    let pool = CanisterPool::load(
        env1,
        false,
        &config.get_config().canisters.as_ref().unwrap().keys().map(|k| k.to_string()).collect::<Vec<String>>(), // hack // FIXME: `unwrap`
    )?;
    builder.read_all_dependencies(
        env1,
        &pool,
        env.get_cache().as_ref(),
    )?;

    Ok(())
}
