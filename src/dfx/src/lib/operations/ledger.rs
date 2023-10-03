use crate::lib::ledger_types::{AccountIdBlob, BlockHeight, Memo, TransferError};
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::{
    error::DfxResult,
    ledger_types::{
        AccountBalanceArgs, IcpXdrConversionRateCertifiedResponse, TimeStamp, TransferArgs,
        TransferResult, MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
    },
    nns_types::{account_identifier::AccountIdentifier, icpts::ICPTs},
};
use anyhow::{bail, ensure, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::{Decode, Encode, Principal};
use fn_error_context::context;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{
    hash_tree::{HashTree, LookupResult},
    lookup_value, Agent, AgentError,
};
use ic_utils::{call::SyncCall, Canister};
use std::time::{SystemTime, UNIX_EPOCH};

const ACCOUNT_BALANCE_METHOD: &str = "account_balance_dfx";
const TRANSFER_METHOD: &str = "transfer";

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
        .query(ACCOUNT_BALANCE_METHOD)
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
        .query("get_icp_xdr_conversion_rate")
        .build()
        .call()
        .await?;
    // check certificate, this is a query call
    let cert = serde_cbor::from_slice(&certified_rate.certificate)?;
    agent
        .verify(&cert, MAINNET_CYCLE_MINTER_CANISTER_ID)
        .context(
            "The origin of the certificate for the XDR <> ICP exchange rate could not be verified",
        )?;
    // we can trust the certificate
    let witness = lookup_value(
        &cert,
        [
            b"canister",
            MAINNET_CYCLE_MINTER_CANISTER_ID.as_slice(),
            b"certified_data",
        ],
    )
    .context("The IC's certificate for the XDR <> ICP exchange rate could not be verified")?;
    let tree = serde_cbor::from_slice::<HashTree<Vec<u8>>>(&certified_rate.hash_tree)?;
    ensure!(
        tree.digest() == witness,
        "The CMC's certificate for the XDR <> ICP exchange rate did not match the IC's certificate"
    );
    // we can trust the hash tree
    let lookup = tree.lookup_path([b"ICP_XDR_CONVERSION_RATE"]);
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

#[context("Failed to transfer funds.")]
pub async fn transfer(
    agent: &Agent,
    canister_id: &Principal,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    from_subaccount: Option<Subaccount>,
    to: AccountIdBlob,
    created_at_time: Option<u64>,
) -> DfxResult<BlockHeight> {
    let timestamp_nanos = created_at_time.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let mut retry_policy = ExponentialBackoff::default();

    let block_height: BlockHeight = loop {
        match agent
            .update(canister_id, TRANSFER_METHOD)
            .with_arg(
                Encode!(&TransferArgs {
                    memo,
                    amount,
                    fee,
                    from_subaccount,
                    to,
                    created_at_time: Some(TimeStamp { timestamp_nanos }),
                })
                .context("Failed to encode arguments.")?,
            )
            .call_and_wait()
            .await
        {
            Ok(data) => {
                let result = Decode!(&data, TransferResult)
                    .context("Failed to decode transfer response.")?;
                match result {
                    Ok(block_height) => break block_height,
                    Err(TransferError::TxDuplicate { duplicate_of }) => {
                        println!("{}", TransferError::TxDuplicate { duplicate_of });
                        break duplicate_of;
                    }
                    Err(transfer_err) => bail!(transfer_err),
                }
            }
            Err(agent_err) if !retryable(&agent_err) => {
                bail!(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => {
                    eprintln!("Waiting to retry after error: {:?}", &agent_err);
                    tokio::time::sleep(duration).await;
                    println!("Sending duplicate transaction");
                }
                None => bail!(agent_err),
            },
        }
    };

    println!("Transfer sent at block height {block_height}");

    Ok(block_height)
}

fn retryable(agent_error: &AgentError) -> bool {
    match agent_error {
        AgentError::ReplicaError(RejectResponse {
            reject_code: RejectCode::CanisterError,
            reject_message,
            ..
        }) if reject_message.contains("is out of cycles") => false,
        AgentError::HttpError(HttpErrorPayload {
            status,
            content_type: _,
            content: _,
        }) if *status == 403 => {
            // sometimes out of cycles looks like this
            // assume any 403(unauthorized) is not retryable
            false
        }
        _ => true,
    }
}
