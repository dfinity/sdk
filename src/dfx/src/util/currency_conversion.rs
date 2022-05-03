use crate::{
    config::dfinity::DEFAULT_IC_GATEWAY,
    lib::{
        error::DfxResult,
        ledger_types::MAINNET_CYCLE_MINTER_CANISTER_ID,
        nns_types::icpts::{ICPTs, ICP_SUBDIVIDABLE_BY},
    },
};
use anyhow::Context;
use candid::{CandidType, Decode, Deserialize, Encode};
use fn_error_context::context;
use ic_agent::{agent::http_transport::ReqwestHttpReplicaV2Transport, Agent};
use serde::Serialize;
use std::convert::TryFrom;

/// How many cycles you get per XDR when converting ICP to cycles
const CYCLES_PER_XDR: u128 = 1_000_000_000_000;

/// This returns how many cycles the amount of ICP/e8s is currently worth.
/// Fetches the exchange rate from the (hardcoded) IC network.
#[context("Encountered a problem while fetching the exchange rate between ICP and cycles. If this issue continues to happen, please specify an amount in cycles directly.")]
pub async fn as_cycles_with_current_exchange_rate(icpts: &ICPTs) -> DfxResult<u128> {
    let cycles_per_icp: u128 = {
        let agent = Agent::builder()
            .with_transport(
                ReqwestHttpReplicaV2Transport::create(DEFAULT_IC_GATEWAY)
                    .context("Failed to create transport object to default ic gateway.")?,
            )
            .build()
            .context("Cannot create mainnet agent.")?;
        let response = agent
            .query(
                &MAINNET_CYCLE_MINTER_CANISTER_ID,
                "get_icp_xdr_conversion_rate",
            )
            .with_arg(Encode!(&()).unwrap())
            .call()
            .await
            .context("Failed to fetch ICP -> cycles conversion rate from mainnet CMC.")?;

        /// These two data structures are stolen straight from https://github.com/dfinity/ic/blob/master/rs/nns/cmc/src/lib.rs
        /// At the time of writing, the latest version is https://github.com/dfinity/ic/blob/d69f7f5b6682958bfdc4836ca4adfa83ce3d4252/rs/nns/cmc/src/lib.rs
        #[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq, Eq)]
        pub struct IcpXdrConversionRateCertifiedResponse {
            pub data: IcpXdrConversionRate,
            pub hash_tree: Vec<u8>,
            pub certificate: Vec<u8>,
        }
        #[derive(Serialize, Deserialize, CandidType, Clone, PartialEq, Eq, Debug, Default)]
        pub struct IcpXdrConversionRate {
            /// The time for which the market data was queried, expressed in UNIX epoch
            /// time in seconds.
            pub timestamp_seconds: u64,
            /// The number of 10,000ths of IMF SDR (currency code XDR) that corresponds
            /// to 1 ICP. This value reflects the current market price of one ICP
            /// token. In other words, this value specifies the ICP/XDR conversion
            /// rate to four decimal places.
            pub xdr_permyriad_per_icp: u64,
        }
        let decoded_response: IcpXdrConversionRateCertifiedResponse =
            Decode!(response.as_slice(), IcpXdrConversionRateCertifiedResponse)
                .context("Failed to decode CMC response.")?;

        let cycles_per_icp: u128 = u128::try_from(decoded_response.data.xdr_permyriad_per_icp)
            .context("Encountered an error while translating response into cycles")?
            * (CYCLES_PER_XDR / 10_000);
        DfxResult::<u128>::Ok(cycles_per_icp)
    }?;

    // This will make rounding errors, but that's fine. We just don't want to be wildly inaccurate.
    let cycles_per_e8s = cycles_per_icp / u128::from(ICP_SUBDIVIDABLE_BY);
    let total_cycles = cycles_per_icp * u128::from(icpts.get_icpts())
        + cycles_per_e8s * u128::from(icpts.get_remainder_e8s());
    Ok(total_cycles)
}
