use clap::{Parser, Subcommand};
use dfx_core::config::model::dfinity::{TelemetryState, ToolConfigInterface};
use slog::{info, warn};

use crate::lib::{environment::Environment, error::DfxResult};

/// Changes settings in dfx's configuration.
#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct ConfigOpts {
    #[command(subcommand)]
    option: ConfigOption,
}

#[derive(Subcommand)]
enum ConfigOption {
    /// Gets or sets whether telemetry is enabled.
    ///
    /// `local` collects telemetry but does not store it.
    Telemetry { telemetry: Option<TelemetryState> },
}

pub fn exec(env: &dyn Environment, opts: ConfigOpts) -> DfxResult {
    match opts.option {
        ConfigOption::Telemetry { telemetry } => {
            if let Some(telemetry) = telemetry {
                update_config(env, |settings| settings.telemetry = telemetry)?;
                info!(env.get_logger(), "Telemetry set to {telemetry}");
                if env.telemetry_mode() != telemetry {
                    warn!(env.get_logger(), "Overridden by environment variable")
                }
            } else {
                println!("{}", env.telemetry_mode());
            }
        }
    }
    Ok(())
}

fn update_config<T>(
    env: &dyn Environment,
    f: impl FnOnce(&mut ToolConfigInterface) -> T,
) -> DfxResult<T> {
    let cfg = env.get_tool_config();
    let mut cfg = cfg.lock().unwrap();
    let settings = cfg.interface_mut();
    let res = f(settings);
    cfg.save()?;
    Ok(res)
}
