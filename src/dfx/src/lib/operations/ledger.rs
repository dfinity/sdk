use anyhow::{bail, ensure, Context};
use candid::{Encode, Principal};
use ic_agent::{hash_tree::LookupResult, ic_types::HashTree, lookup_value, Agent};
use ic_utils::{call::SyncCall, Canister};

use crate::lib::{
    error::DfxResult,
    ledger_types::{
        AccountBalanceArgs, IcpXdrConversionRateCertifiedResponse,
        MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
    },
    nns_types::{account_identifier::AccountIdentifier, icpts::ICPTs},
};

const ACCOUNT_BALANCE_METHOD: &str = "account_balance_dfx";

pub async fn balance(
    agent: &Agent,
    acct: &AccountIdentifier,
    ledger_canister_id: Option<Principal>,
) -> DfxResult<ICPTs> {
    let canister_id = ledger_canister_id.unwrap_or(MAINNET_LEDGER_CANISTER_ID);
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(canister_id)
        .build()?;
    let (result,) = canister
        .query_(ACCOUNT_BALANCE_METHOD)
        .with_arg(AccountBalanceArgs {
            account: acct.to_string(),
        })
        .build()
        .call()
        .await?;
    Ok(result)
}

/// Returns XDR-permyriad (i.e. ten-thousandths-of-an-XDR) per ICP.
pub async fn xdr_permyriad_per_icp(agent: &Agent) -> DfxResult<u64> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(MAINNET_CYCLE_MINTER_CANISTER_ID)
        .build()?;
    let (certified_rate,): (IcpXdrConversionRateCertifiedResponse,) = canister
        .query_("get_icp_xdr_conversion_rate")
        .build()
        .call()
        .await?;
    // check certificate, this is a query call
    let cert = serde_cbor::from_slice(&certified_rate.certificate)?;
    agent
        .verify(&cert, MAINNET_CYCLE_MINTER_CANISTER_ID, false)
        .context(
            "The origin of the certificate for the XDR <> ICP exchange rate could not be verified",
        )?;
    // we can trust the certificate
    let witness = lookup_value(
        &cert,
        [
            "canister".into(),
            MAINNET_CYCLE_MINTER_CANISTER_ID.into(),
            "certified_data".into(),
        ],
    )
    .context("The IC's certificate for the XDR <> ICP exchange rate could not be verified")?;
    let tree = serde_cbor::from_slice::<HashTree>(&certified_rate.hash_tree)?;
    ensure!(
        tree.digest() == witness,
        "The CMC's certificate for the XDR <> ICP exchange rate did not match the IC's certificate"
    );
    // we can trust the hash tree
    let lookup = tree.lookup_path([&"ICP_XDR_CONVERSION_RATE".into()]);
    let certified_data = if let LookupResult::Found(content) = lookup {
        content
    } else {
        bail!("The CMC's certificate did not contain the XDR <> ICP exchange rate");
    };
    let encoded_data = Encode!(&certified_rate.data)?;
    ensure!(
        certified_data == encoded_data,
        "The CMC's certificate for the XDR <> ICP exchange rate did not match the provided rate"
    );
    // we can trust the exchange rate
    Ok(certified_rate.data.xdr_permyriad_per_icp)
}
