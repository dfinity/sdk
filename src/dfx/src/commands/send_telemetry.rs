use crate::lib::error::DfxResult;
use crate::lib::telemetry::Telemetry;
use clap::Parser;
use url::Url;

const DEFAULT_URL: &str = "https://sdk.telemetry.dfinity.network";

#[derive(Parser)]
#[command(hide = true)]
pub struct SendTelemetryOpts {
    #[clap(long)]
    url: Option<String>,
}

pub fn exec(opts: SendTelemetryOpts) -> DfxResult {
    let url = opts.url.unwrap_or_else(|| DEFAULT_URL.to_string());
    let url = Url::parse(&url)?;

    Telemetry::send(&url)
}
