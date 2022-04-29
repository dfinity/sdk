use crate::{
    lib::{
        error::DfxResult,
        nns_types::icpts::{ICPTs, ICP_SUBDIVIDABLE_BY}, operations::ledger::icp_xdr_rate,
    },
};
use anyhow::Context;
use ic_agent::Agent;
use std::convert::TryFrom;

/// How many cycles you get per XDR when converting ICP to cycles
const CYCLES_PER_XDR: u128 = 1_000_000_000_000;

/// This returns how many cycles the amount of ICP/e8s is currently worth.
/// Fetches the exchange rate from the (hardcoded) IC network.
pub async fn as_cycles_with_current_exchange_rate(agent: &Agent, icpts: &ICPTs) -> DfxResult<u128> {
    let cycles_per_icp: u128 = {
        let xdr_permyriad_per_icp = icp_xdr_rate(agent).await
            .context("Failed to fetch ICP -> cycles conversion rate from mainnet CMC.")?;

        let cycles_per_icp: u128 = u128::try_from(xdr_permyriad_per_icp).context("Encountered an error while translating response into cycles")? * (CYCLES_PER_XDR / 10_000);
        DfxResult::<u128>::Ok(cycles_per_icp)
    }.context("Encountered a problem while fetching the exchange rate between ICP and cycles. If this issue continues to happen, please specify an amount in cycles directly.")?;

    // This will make rounding errors, but that's fine. We just don't want to be wildly inaccurate.
    let cycles_per_e8s = cycles_per_icp / u128::from(ICP_SUBDIVIDABLE_BY);
    let total_cycles = cycles_per_icp * u128::from(icpts.get_icpts())
        + cycles_per_e8s * u128::from(icpts.get_remainder_e8s());
    Ok(total_cycles)
}
