use crate::lib::error::{
    DfxResult, NotifyCreateCanisterError, NotifyMintCyclesError, NotifyTopUpError,
};
use crate::lib::ledger_types::{
    BlockHeight, BlockIndex, Memo, NotifyCreateCanisterArg, NotifyCreateCanisterResult,
    NotifyMintCyclesArg, NotifyMintCyclesResult, NotifyMintCyclesSuccess, NotifyTopUpArg,
    NotifyTopUpResult, MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::operations::ledger::transfer;
use crate::util::clap::subnet_selection_opt::SubnetSelectionType;
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;
use icrc_ledger_types::icrc1::account::Subaccount as ICRCSubaccount;
use icrc_ledger_types::icrc1::transfer::Memo as ICRCMemo;
use slog::Logger;

const NOTIFY_CREATE_CANISTER_METHOD: &str = "notify_create_canister";
const NOTIFY_TOP_UP_METHOD: &str = "notify_top_up";
const NOTIFY_MINT_CYCLES_METHOD: &str = "notify_mint_cycles";

pub async fn transfer_cmc(
    agent: &Agent,
    logger: &Logger,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    from_subaccount: Option<Subaccount>,
    to_principal: Principal,
    created_at_time: Option<u64>,
) -> DfxResult<BlockHeight> {
    let to_subaccount = Subaccount::from(&to_principal);
    let to =
        AccountIdentifier::new(MAINNET_CYCLE_MINTER_CANISTER_ID, Some(to_subaccount)).to_address();
    transfer(
        agent,
        logger,
        &MAINNET_LEDGER_CANISTER_ID,
        memo,
        amount,
        fee,
        from_subaccount,
        to,
        created_at_time,
    )
    .await
}

pub async fn notify_create(
    agent: &Agent,
    controller: Principal,
    block_height: BlockHeight,
    subnet_selection: SubnetSelectionType,
) -> Result<Principal, NotifyCreateCanisterError> {
    let user_subnet_selection = subnet_selection.get_user_choice();
    let result = agent
        .update(
            &MAINNET_CYCLE_MINTER_CANISTER_ID,
            NOTIFY_CREATE_CANISTER_METHOD,
        )
        .with_arg(
            Encode!(&NotifyCreateCanisterArg {
                block_index: block_height,
                controller,
                subnet_selection: user_subnet_selection,
            })
            .map_err(NotifyCreateCanisterError::EncodeArguments)?,
        )
        .call_and_wait()
        .await
        .map_err(NotifyCreateCanisterError::Call)?;
    Decode!(&result, NotifyCreateCanisterResult)
        .map_err(NotifyCreateCanisterError::DecodeResponse)?
        .map_err(NotifyCreateCanisterError::Notify)
}

pub async fn notify_top_up(
    agent: &Agent,
    canister: Principal,
    block_height: BlockHeight,
) -> Result<u128, NotifyTopUpError> {
    let result = agent
        .update(&MAINNET_CYCLE_MINTER_CANISTER_ID, NOTIFY_TOP_UP_METHOD)
        .with_arg(
            Encode!(&NotifyTopUpArg {
                block_index: block_height,
                canister_id: canister,
            })
            .map_err(NotifyTopUpError::EncodeArguments)?,
        )
        .call_and_wait()
        .await
        .map_err(NotifyTopUpError::Call)?;
    Decode!(&result, NotifyTopUpResult)
        .map_err(NotifyTopUpError::DecodeResponse)?
        .map_err(NotifyTopUpError::Notify)
}

pub async fn notify_mint_cycles(
    agent: &Agent,
    deposit_memo: Option<ICRCMemo>,
    to_subaccount: Option<ICRCSubaccount>,
    block_index: BlockIndex,
) -> Result<NotifyMintCyclesSuccess, NotifyMintCyclesError> {
    let result = agent
        .update(&MAINNET_CYCLE_MINTER_CANISTER_ID, NOTIFY_MINT_CYCLES_METHOD)
        .with_arg(
            Encode!(&NotifyMintCyclesArg {
                block_index,
                to_subaccount,
                deposit_memo: deposit_memo.map(|memo| memo.0.as_ref().into()),
            })
            .map_err(NotifyMintCyclesError::EncodeArguments)?,
        )
        .call_and_wait()
        .await
        .map_err(NotifyMintCyclesError::Call)?;
    Decode!(&result, NotifyMintCyclesResult)
        .map_err(NotifyMintCyclesError::DecodeResponse)?
        .map_err(NotifyMintCyclesError::Notify)
}
