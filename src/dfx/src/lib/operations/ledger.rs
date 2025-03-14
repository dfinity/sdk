use crate::lib::diagnosis::DiagnosedError;
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
use anyhow::{anyhow, bail, ensure, Context};
use backoff::backoff::Backoff;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use candid::{Decode, Encode, Nat, Principal};
use fn_error_context::context;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{
    hash_tree::{HashTree, LookupResult},
    lookup_value, Agent, AgentError,
};
use ic_utils::{call::SyncCall, Canister};
use icrc_ledger_types::icrc1;
use icrc_ledger_types::icrc1::transfer::BlockIndex;
use icrc_ledger_types::icrc2;
use icrc_ledger_types::icrc2::allowance::Allowance;
use icrc_ledger_types::icrc2::approve::ApproveError;
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;
use slog::{info, Logger};
use std::time::{SystemTime, UNIX_EPOCH};

const ACCOUNT_BALANCE_METHOD: &str = "account_balance_dfx";
const TRANSFER_METHOD: &str = "transfer";
const ICRC2_APPROVE_METHOD: &str = "icrc2_approve";
const ICRC2_TRANSFER_FROM_METHOD: &str = "icrc2_transfer_from";
const ICRC2_ALLOWANCE_METHOD: &str = "icrc2_allowance";

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
    logger: &Logger,
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
            .await
        {
            Ok(data) => {
                let result = Decode!(&data, TransferResult)
                    .context("Failed to decode transfer response.")?;
                match result {
                    Ok(block_height) => break block_height,
                    Err(TransferError::TxDuplicate { duplicate_of }) => {
                        info!(logger, "{}", TransferError::TxDuplicate { duplicate_of });
                        break duplicate_of;
                    }
                    Err(TransferError::InsufficientFunds { balance }) => {
                        return Err(anyhow!(TransferError::InsufficientFunds { balance }))
                            .with_context(|| {
                                diagnose_insufficient_funds_error(agent, from_subaccount)
                            });
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

pub async fn transfer_from(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    spender_subaccount: Option<icrc1::account::Subaccount>,
    from: icrc1::account::Account,
    to: icrc1::account::Account,
    amount: ICPTs,
    fee: Option<ICPTs>,
    created_at_time: u64,
    memo: Option<u64>,
) -> DfxResult<BlockIndex> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(*canister_id)
        .build()?;

    let retry_policy = ExponentialBackoff::default();

    let block_index = retry(retry_policy, || async {
        let arg = icrc2::transfer_from::TransferFromArgs {
            spender_subaccount,
            from,
            to,
            fee: fee.map(|value| Nat::from(value.get_e8s())),
            created_at_time: Some(created_at_time),
            memo: memo.map(|v| v.into()),
            amount: Nat::from(amount.get_e8s()),
        };
        match canister
            .update(ICRC2_TRANSFER_FROM_METHOD)
            .with_arg(arg)
            .build()
            .map(|result: (Result<BlockIndex, TransferFromError>,)| (result.0,))
            .await
            .map(|(result,)| result)
        {
            Ok(Ok(block_index)) => Ok(block_index),
            Ok(Err(TransferFromError::Duplicate { duplicate_of })) => {
                info!(
                    logger,
                    "Transfer is a duplicate of block index {}", duplicate_of
                );
                Ok(duplicate_of)
            }
            Ok(Err(transfer_from_err)) => {
                Err(backoff::Error::permanent(anyhow!(transfer_from_err)))
            }
            Err(agent_err) if retryable(&agent_err) => {
                Err(backoff::Error::transient(anyhow!(agent_err)))
            }
            Err(agent_err) => Err(backoff::Error::permanent(anyhow!(agent_err))),
        }
    })
    .await?;

    Ok(block_index)
}

pub async fn approve(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    from_subaccount: Option<icrc1::account::Subaccount>,
    spender: Principal,
    spender_subaccount: Option<icrc1::account::Subaccount>,
    amount: ICPTs,
    expected_allowance: Option<ICPTs>,
    fee: Option<ICPTs>,
    created_at_time: u64,
    expires_at: Option<u64>,
    memo: Option<u64>,
) -> DfxResult<BlockIndex> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(*canister_id)
        .build()?;

    let retry_policy = ExponentialBackoff::default();

    let block_index = retry(retry_policy, || async {
        let arg = icrc2::approve::ApproveArgs {
            from_subaccount,
            fee: fee.map(|value| Nat::from(value.get_e8s())),
            created_at_time: Some(created_at_time),
            memo: memo.map(|v| v.into()),
            amount: Nat::from(amount.get_e8s()),
            spender: icrc1::account::Account {
                owner: spender,
                subaccount: spender_subaccount,
            },
            expected_allowance: expected_allowance.map(|value| Nat::from(value.get_e8s())),
            expires_at,
        };
        match canister
            .update(ICRC2_APPROVE_METHOD)
            .with_arg(arg)
            .build()
            .map(|result: (Result<BlockIndex, ApproveError>,)| (result.0,))
            .await
            .map(|(result,)| result)
        {
            Ok(Ok(block_index)) => Ok(block_index),
            Ok(Err(ApproveError::Duplicate { duplicate_of })) => {
                info!(logger, "Approval is a duplicate of block {}", duplicate_of);
                Ok(duplicate_of)
            }
            Ok(Err(approve_err)) => Err(backoff::Error::permanent(anyhow!(approve_err))),
            Err(agent_err) if retryable(&agent_err) => {
                Err(backoff::Error::transient(anyhow!(agent_err)))
            }
            Err(agent_err) => Err(backoff::Error::permanent(anyhow!(agent_err))),
        }
    })
    .await?;

    Ok(block_index)
}

pub async fn allowance(
    agent: &Agent,
    canister_id: &Principal,
    owner: icrc1::account::Account,
    spender: icrc1::account::Account,
) -> DfxResult<Allowance> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(*canister_id)
        .build()?;

    let retry_policy = ExponentialBackoff::default();

    retry(retry_policy, || async {
        let arg = icrc2::allowance::AllowanceArgs {
            account: owner,
            spender,
        };
        let result = canister
            .query(ICRC2_ALLOWANCE_METHOD)
            .with_arg(arg)
            .build()
            .call()
            .await;
        match result {
            Ok((allowance,)) => Ok(allowance),
            Err(agent_err) if retryable(&agent_err) => {
                Err(backoff::Error::transient(anyhow!(agent_err)))
            }
            Err(agent_err) => Err(backoff::Error::permanent(anyhow!(agent_err))),
        }
    })
    .await
}

fn diagnose_insufficient_funds_error(
    agent: &Agent,
    subaccount: Option<Subaccount>,
) -> DiagnosedError {
    let principal = agent.get_principal().unwrap(); // This should always succeed at this point.

    let explanation = "Insufficient ICP balance to finish the transfer transaction.";
    let suggestion = format!(
        "Please top up your ICP balance.

Your account address for receiving ICP from centralized exchanges: {}
(run `dfx ledger account-id` to display)

Your principal for ICP wallets and decentralized exchanges: {}
(run `dfx identity get-principal` to display)
",
        AccountIdentifier::new(principal, subaccount),
        principal.to_text()
    );

    DiagnosedError::new(explanation, suggestion)
}

fn retryable(agent_error: &AgentError) -> bool {
    match agent_error {
        AgentError::CertifiedReject(RejectResponse {
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
