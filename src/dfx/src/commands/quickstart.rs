use anyhow::Error;
use ic_agent::Identity;
use num_traits::Inv;
use rust_decimal::Decimal;
use tokio::runtime::Runtime;

use crate::lib::{
    environment::Environment, error::DfxResult, identity::IdentityManager,
    nns_types::account_identifier::AccountIdentifier, operations::ledger::{balance, icp_xdr_rate}, provider::create_agent_environment,
};

pub fn exec(env: &dyn Environment) -> DfxResult {
    let env = create_agent_environment(env, Some("ic".to_string()))?;
    let agent = env.get_agent().expect("Unable to create agent");
    let ident = IdentityManager::new(&env)?.instantiate_selected_identity()?;
    let principal = ident.sender().map_err(Error::msg)?;
    println!("Your DFX user principal: {principal}");
    let acct = AccountIdentifier::new(principal, None);
    println!("Your ledger account ID: {acct}");
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        let balance = balance(agent, &acct, None).await?;
        println!("Your ICP balance: {balance}");
        let xdr_conversion_rate = icp_xdr_rate(agent).await?;
        let xdr_per_icp = Decimal::from_i128_with_scale(xdr_conversion_rate as i128, 4);
        let _icp_per_tc = xdr_per_icp.inv();
        println!("Conversion rate: 1 ICP <> {xdr_per_icp} XDR");
        // todo integrate the wallet
        Ok(())
    })
}
