use anyhow::{bail, Context, Error};
use candid::Principal;
use dialoguer::{Confirm, Input};
use ic_agent::Identity as _;
use ic_utils::interfaces::{
    management_canister::builders::InstallMode, ManagementCanister, WalletCanister,
};
use indicatif::ProgressBar;
use num_traits::Inv;
use rust_decimal::Decimal;
use tokio::runtime::Runtime;

use crate::{
    commands::ledger::{create_canister::MEMO_CREATE_CANISTER, notify_create, transfer_cmc},
    lib::{
        environment::Environment,
        error::DfxResult,
        identity::{Identity, IdentityManager},
        ledger_types::{Memo, NotifyError},
        nns_types::{
            account_identifier::AccountIdentifier,
            icpts::{ICPTs, TRANSACTION_FEE},
        },
        operations::{
            canister::install_wallet,
            ledger::{balance, xdr_permyriad_per_icp},
        },
        provider::create_agent_environment,
        waiter::waiter_with_timeout,
    },
    util::{assets::wallet_wasm, expiry_duration},
};

pub fn exec(env: &dyn Environment) -> DfxResult {
    let env = create_agent_environment(env, Some("ic".to_string()))?;
    let agent = env.get_agent().expect("Unable to create agent");
    let ident = IdentityManager::new(&env)?.instantiate_selected_identity()?;
    let principal = ident.sender().map_err(Error::msg)?;
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
        let wallet = Identity::wallet_canister_id(env.get_network_descriptor(), ident.name())?;
        if let Some(wallet) = wallet {
            eprintln!("Mainnet wallet canister: {wallet}");
            if let Ok(wallet_canister) = WalletCanister::create(agent, wallet).await {
                if let Ok(balance) = wallet_canister.wallet_balance().await {
                    eprintln!("Mainnet wallet balance: {:.2} TC", Decimal::from(balance.amount) / Decimal::from(1_000_000_000_000_u64));
                }
            }
        } else if Confirm::new().with_prompt("Import an existing wallet?").interact()? {
            let id = Input::<Principal>::new().with_prompt("Paste the principal ID of the existing wallet").interact_text()?;
            let wallet = if let Ok(wallet) = WalletCanister::create(agent, id).await {
                wallet
            } else {
                let mgmt = ManagementCanister::create(agent);
                let wasm = wallet_wasm(env.get_logger())?;
                mgmt.install_code(&id, &wasm).with_mode(InstallMode::Install).call_and_wait(waiter_with_timeout(expiry_duration())).await?;
                WalletCanister::create(agent, id).await?
            };
            Identity::set_wallet_id( env.get_network_descriptor(), ident.name(), id)?;
            eprintln!("Successfully imported wallet {id}.");
            if let Ok(balance) = wallet.wallet_balance().await {
                eprintln!("Mainnet wallet balance: {:.2} TC", Decimal::from(balance.amount) / Decimal::from(1_000_000_000_000_u64));
            }
        } else {
            let possible_tc = xdr_per_icp * balance.to_decimal();
            let needed_tc = Decimal::new(10, 0) - possible_tc;
            if needed_tc.is_sign_positive() {
                let needed_icp = needed_tc * icp_per_tc;
                let rounded = needed_icp.round_dp(8);
                eprintln!("\nYou need {rounded:.8} more ICP to deploy a 10 TC wallet canister on mainnet.");
                eprintln!("Deposit at least {rounded:.8} ICP into the address {acct}, and then run this command again, to deploy a mainnet wallet.");
                eprintln!("\nAlternatively:");
                eprintln!("- If you have ICP in an NNS account, you can create a new canister through the NNS interface");
                eprintln!("- If you have a Discord account, you can request free cycles at https://faucet.dfinity.org");
                eprintln!("Either of these options will ask for your DFX user principal, listed above.");
                eprintln!("And either of these options will hand you back a wallet canister principal; when you run the command again, select the 'import an existing wallet' option.");
            } else {
                let to_spend = Decimal::new(10, 0) * icp_per_tc;
                let rounded = to_spend.round_dp(8);
                if Confirm::new().with_prompt(format!("Spend {rounded:.8} ICP to create a new wallet with 10 TC?")).interact()? {
                    let send_spinner = ProgressBar::new_spinner();
                    send_spinner.set_message(format!("Sending {rounded:.8} ICP to the cycles minting canister..."));
                    send_spinner.enable_steady_tick(100);
                    let icpts = ICPTs::from_decimal(rounded)?;
                    let height = transfer_cmc(agent, Memo(MEMO_CREATE_CANISTER /* 👽 */), icpts, TRANSACTION_FEE, None, principal).await
                        .context("Failed to transfer to the cycles minting canister")?;
                    send_spinner.finish_with_message(format!("Sent {icpts} to the cycles minting canister at height {height}"));
                    let notify_spinner = ProgressBar::new_spinner();
                    notify_spinner.set_message("Notifying the the cycles minting canister...");
                    notify_spinner.enable_steady_tick(100);
                    let res = notify_create(agent, principal, height).await
                        .with_context(|| format!("Failed to notify the CMC of the transfer. Write down that height ({height}), and once the error is fixed, use `dfx ledger notify create-canister`."))?;
                    let wallet = match res {
                        Ok(principal) => principal,
                        Err(NotifyError::Refunded { reason, block_index }) => {
                            match block_index {
                                Some(height) => {
                                    bail!("Refunded at block height {height} with message: {reason}")
                                }
                                None => bail!("Refunded with message: {reason}"),
                            };
                        }
                        Err(err) => bail!("{err:?}"),
                    };
                    notify_spinner.finish_with_message(format!("Created wallet canister with principal ID {wallet}"));
                    let install_spinner = ProgressBar::new_spinner();
                    install_spinner.set_message("Installing the wallet code to the canister...");
                    install_spinner.enable_steady_tick(100);
                    install_wallet(&env, agent, wallet, InstallMode::Install).await.context("Failed to install the wallet code to the canister")?;
                    Identity::set_wallet_id(env.get_network_descriptor(), ident.name(), wallet).context("Failed to record the wallet's principal as your associated wallet")?;
                    install_spinner.finish_with_message("Installed the wallet code to the canister");
                    eprintln!("Success! Run this command again at any time to print all this information again.");
                } else {
                    eprintln!("Run this command again at any time to continue from here."); // unify somehow
                    return Ok(());
                }
            }
        }
        Ok(())
    })
}
