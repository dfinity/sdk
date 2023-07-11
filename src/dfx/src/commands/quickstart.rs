use anyhow::{bail, Context};
use candid::Principal;
use clap::Parser;
use dialoguer::{Confirm, Input};
use ic_agent::Agent;
use ic_utils::interfaces::{
    management_canister::builders::InstallMode, ManagementCanister, WalletCanister,
};
use indicatif::ProgressBar;
use num_traits::Inv;
use rust_decimal::Decimal;
use tokio::runtime::Runtime;

use crate::{
    commands::ledger::create_canister::MEMO_CREATE_CANISTER,
    lib::{
        agent::create_agent_environment,
        environment::Environment,
        error::DfxResult,
        identity::wallet::{set_wallet_id, wallet_canister_id},
        ledger_types::{Memo, NotifyError},
        nns_types::{
            account_identifier::AccountIdentifier,
            icpts::{ICPTs, TRANSACTION_FEE},
        },
        operations::{
            canister::install_wallet,
            cmc::{notify_create, transfer_cmc},
            ledger::{balance, xdr_permyriad_per_icp},
        },
    },
    util::assets::wallet_wasm,
};

/// Use the `dfx quickstart` command to perform initial one time setup for your identity and/or wallet. This command
/// can be run anytime to repeat the setup process or to be used as an informational command, printing
/// information about your ICP balance, current ICP to XDR conversion rate, and more.
///
/// If setup tasks remain, this command is equivalent to running `dfx identity set-wallet` (for importing) or
/// `dfx ledger create-canister` followed by `dfx identity deploy-wallet` (for creating), though more steps may
/// be added in future versions.
#[derive(Parser)]
pub struct QuickstartOpts;

pub fn exec(env: &dyn Environment, _: QuickstartOpts) -> DfxResult {
    let env = create_agent_environment(env, Some("ic".to_string()))?;
    let agent = env.get_agent().expect("Unable to create agent");
    let ident = env.get_selected_identity().unwrap();
    let principal = env.get_selected_identity_principal().unwrap();
    eprintln!("Your DFX user principal: {principal}");
    let acct = AccountIdentifier::new(principal, None);
    eprintln!("Your ledger account address: {acct}");
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        let balance = balance(agent, &acct, None).await?;
        eprintln!("Your ICP balance: {balance}");
        let xdr_conversion_rate = xdr_permyriad_per_icp(agent).await?;
        let xdr_per_icp = Decimal::from_i128_with_scale(xdr_conversion_rate as i128, 4);
        let icp_per_tc = xdr_per_icp.inv();
        eprintln!("Conversion rate: 1 ICP <> {xdr_per_icp} XDR");
        let wallet = wallet_canister_id(env.get_network_descriptor(), ident)?;
        if let Some(wallet) = wallet {
            step_print_wallet(agent, wallet).await?;
        } else if Confirm::new()
            .with_prompt("Import an existing wallet?")
            .interact()?
        {
            step_import_wallet(&env, agent, ident).await?;
        } else {
            step_deploy_wallet(
                &env,
                agent,
                acct,
                ident,
                principal,
                balance.to_decimal(),
                xdr_per_icp,
                icp_per_tc,
            )
            .await?;
        }
        Ok(())
    })
}

async fn step_print_wallet(agent: &Agent, wallet: Principal) -> DfxResult {
    eprintln!("Mainnet wallet canister: {wallet}");
    if let Ok(wallet_canister) = WalletCanister::create(agent, wallet).await {
        if let Ok(balance) = wallet_canister.wallet_balance().await {
            eprintln!(
                "Mainnet wallet balance: {:.2} TC",
                Decimal::from(balance.amount) / Decimal::from(1_000_000_000_000_u64)
            );
        }
    }
    Ok(())
}

async fn step_import_wallet(env: &dyn Environment, agent: &Agent, ident: &str) -> DfxResult {
    let id = Input::<Principal>::new()
        .with_prompt("Paste the principal ID of the existing wallet")
        .interact_text()?;
    let wallet = if let Ok(wallet) = WalletCanister::create(agent, id).await {
        wallet
    } else {
        let mgmt = ManagementCanister::create(agent);
        let wasm = wallet_wasm(env.get_logger())?;
        mgmt.install_code(&id, &wasm)
            .with_mode(InstallMode::Install)
            .call_and_wait()
            .await?;
        WalletCanister::create(agent, id).await?
    };
    set_wallet_id(env.get_network_descriptor(), ident, id)?;
    eprintln!("Successfully imported wallet {id}.");
    if let Ok(balance) = wallet.wallet_balance().await {
        eprintln!(
            "Mainnet wallet balance: {:.2} TC",
            Decimal::from(balance.amount) / Decimal::from(1_000_000_000_000_u64)
        );
    }
    Ok(())
}

async fn step_deploy_wallet(
    env: &dyn Environment,
    agent: &Agent,
    acct: AccountIdentifier,
    ident: &str,
    ident_principal: Principal,
    balance: Decimal,
    xdr_per_icp: Decimal,
    icp_per_tc: Decimal,
) -> DfxResult {
    let possible_tc = xdr_per_icp * balance;
    let needed_tc = Decimal::new(10, 0) - possible_tc;
    if needed_tc.is_sign_positive() {
        let needed_icp = needed_tc * icp_per_tc;
        step_explain_deploy(acct, needed_icp.round_dp(8));
        return Ok(());
    }
    let to_spend = Decimal::new(10, 0) * icp_per_tc;
    let rounded = to_spend.round_dp(8);
    if !Confirm::new()
        .with_prompt(format!(
            "Spend {rounded:.8} ICP to create a new wallet with 10 TC?"
        ))
        .interact()?
    {
        eprintln!("Run this command again at any time to continue from here.");
        return Ok(());
    }
    let wallet = step_interact_ledger(agent, ident_principal, rounded).await?;
    step_finish_wallet(env, agent, wallet, ident).await?;
    Ok(())
}

async fn step_interact_ledger(
    agent: &Agent,
    ident_principal: Principal,
    to_spend: Decimal,
) -> DfxResult<Principal> {
    let send_spinner = ProgressBar::new_spinner();
    send_spinner.set_message(format!(
        "Sending {to_spend:.8} ICP to the cycles minting canister..."
    ));
    send_spinner.enable_steady_tick(100);
    let icpts = ICPTs::from_decimal(to_spend)?;
    let height = transfer_cmc(
        agent,
        Memo(MEMO_CREATE_CANISTER /* 👽 */),
        icpts,
        TRANSACTION_FEE,
        None,
        ident_principal,
        None,
    )
    .await
    .context("Failed to transfer to the cycles minting canister")?;
    send_spinner.finish_with_message(format!(
        "Sent {icpts} to the cycles minting canister at height {height}"
    ));
    let notify_spinner = ProgressBar::new_spinner();
    notify_spinner.set_message("Notifying the cycles minting canister...");
    notify_spinner.enable_steady_tick(100);
    let res = notify_create(agent, ident_principal, height, None).await
        .with_context(|| format!("Failed to notify the CMC of the transfer. Write down that height ({height}), and once the error is fixed, use `dfx ledger notify create-canister`."))?;
    let wallet = match res {
        Ok(principal) => principal,
        Err(NotifyError::Refunded {
            reason,
            block_index,
        }) => {
            match block_index {
                Some(height) => {
                    bail!("Refunded at block height {height} with message: {reason}")
                }
                None => bail!("Refunded with message: {reason}"),
            };
        }
        Err(err) => bail!("{err:?}"),
    };
    notify_spinner.finish_with_message(format!(
        "Created wallet canister with principal ID {wallet}"
    ));
    Ok(wallet)
}

async fn step_finish_wallet(
    env: &dyn Environment,
    agent: &Agent,
    wallet: Principal,
    ident: &str,
) -> DfxResult {
    let install_spinner = ProgressBar::new_spinner();
    install_spinner.set_message("Installing the wallet code to the canister...");
    install_spinner.enable_steady_tick(100);
    install_wallet(env, agent, wallet, InstallMode::Install)
        .await
        .context("Failed to install the wallet code to the canister")?;
    set_wallet_id(env.get_network_descriptor(), ident, wallet)
        .context("Failed to record the wallet's principal as your associated wallet")?;
    install_spinner.finish_with_message("Installed the wallet code to the canister");
    eprintln!("Success! Run this command again at any time to print all this information again.");
    Ok(())
}

fn step_explain_deploy(acct: AccountIdentifier, needed_icp: Decimal) {
    eprintln!("\nYou need {needed_icp:.8} more ICP to deploy a 10 TC wallet canister on mainnet.");
    eprintln!("Deposit at least {needed_icp:.8} ICP into the address {acct}, and then run this command again, to deploy a mainnet wallet.");
    eprintln!("\nAlternatively:");
    eprintln!("- If you have ICP in an NNS account, you can create a new canister through the NNS interface");
    eprintln!("- If you have a Discord account, you can request free cycles at https://faucet.dfinity.org");
    eprintln!("Either of these options will ask for your DFX user principal, listed above.");
    eprintln!("And either of these options will hand you back a wallet canister principal; when you run the command again, select the 'import an existing wallet' option.");
}
