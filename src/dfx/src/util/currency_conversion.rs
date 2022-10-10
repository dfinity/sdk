use crate::{
    config::dfinity::DEFAULT_IC_GATEWAY,
    lib::{error::DfxResult, nns_types::icpts::ICPTs, operations::ledger::xdr_permyriad_per_icp},
};
use anyhow::Context;
use fn_error_context::context;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, Agent};
use rust_decimal::Decimal;

/// How many cycles you get per XDR when converting ICP to cycles
const CYCLES_PER_XDR: u128 = 1_000_000_000_000;

/// This returns how many cycles the amount of ICP/e8s is currently worth.
/// Fetches the exchange rate from the (hardcoded) IC network.
#[context("Encountered a problem while fetching the exchange rate between ICP and cycles. If this issue continues to happen, please specify an amount in cycles directly.")]
pub async fn as_cycles_with_current_exchange_rate(icpts: &ICPTs) -> DfxResult<u128> {
    let agent = Agent::builder()
        .with_transport(
            ReqwestHttpReplicaV2Transport::create(DEFAULT_IC_GATEWAY)
                .context("Failed to create transport object to default ic gateway.")?,
        )
        .build()
        .context("Cannot create mainnet agent.")?;
    let xdr_permyriad_per_icp = xdr_permyriad_per_icp(&agent).await?;
    let xdr_per_icp = Decimal::from_i128_with_scale(xdr_permyriad_per_icp as i128, 4);
    let xdr = xdr_per_icp * icpts.to_decimal();
    let cycles = xdr * Decimal::from(CYCLES_PER_XDR);
    Ok(u128::try_from(cycles)?)
}
