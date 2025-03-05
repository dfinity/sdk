use clap::Parser;
use dfx_core::config::model::dfinity::TelemetryState;

use crate::lib::{environment::Environment, error::DfxResult};

/// Changes settings in dfx's configuration
#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct ConfigOpts {
    /// Enables or disables telemetry/metrics. Enabled by default. `local` means it is collected but not uploaded.
    #[arg(long)]
    telemetry: Option<TelemetryState>,
}

pub fn exec(env: &dyn Environment, opts: ConfigOpts) -> DfxResult {
    let cfg = env.get_tool_config();
    let mut cfg = cfg.lock().unwrap();
    let settings = cfg.interface_mut();
    if let Some(telemetry) = opts.telemetry {
        settings.telemetry = telemetry;
    }
    cfg.save()?;
    Ok(())
}
