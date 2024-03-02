use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use clap::Parser;
use tokio::runtime::Runtime;

mod approve;
mod balance;
mod convert;
mod redeem_faucet_coupon;
pub mod top_up;
mod transfer;

/// Helper commands to manage the user's cycles.
#[derive(Parser)]
#[command(name = "wallet")]
pub struct CyclesOpts {
    #[command(flatten)]
    network: NetworkOpt,

    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Approve(approve::ApproveOpts),
    Balance(balance::CyclesBalanceOpts),
    Convert(convert::ConvertOpts),
    TopUp(top_up::TopUpOpts),
    Transfer(transfer::TransferOpts),
    RedeemFaucetCoupon(redeem_faucet_coupon::RedeemFaucetCouponOpts),
}

pub fn exec(env: &dyn Environment, opts: CyclesOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.to_network_name())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Approve(v) => approve::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::Convert(v) => convert::exec(&agent_env, v).await,
            SubCommand::TopUp(v) => top_up::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
            SubCommand::RedeemFaucetCoupon(v) => redeem_faucet_coupon::exec(&agent_env, v).await,
        }
    })
}
