use candid::Principal;
use clap::{ArgGroup, Args};

use crate::lib::{
    cycles_ledger_types::create_canister::{SubnetFilter, SubnetSelection},
    environment::Environment,
    error::DfxResult,
    subnet::get_subnet_for_canister,
};

#[derive(Args, Clone, Debug, Default)]
#[clap(
group(ArgGroup::new("subnet-selection").multiple(false)),
)]
pub struct SubnetSelectionOpt {
    /// Specify the optional subnet type to create canisters on. If no
    /// subnet type is provided, the canister will be created on a random
    /// default application subnet.
    #[arg(long, group = "subnet-selection")]
    subnet_type: Option<String>,

    /// Specify a specific subnet on which to create canisters on.
    #[arg(long, group = "subnet-selection")]
    subnet: Option<Principal>,

    /// Create canisters on the same subnet as this canister.
    #[arg(long, group = "subnet-selection")]
    next_to: Option<String>,
}

impl SubnetSelectionOpt {
    pub async fn into_subnet_selection(
        self,
        env: &dyn Environment,
    ) -> DfxResult<Option<SubnetSelection>> {
        if let Some(sibling) = self.next_to {
            let next_to = Principal::from_text(&sibling)
                .or_else(|_| env.get_canister_id_store()?.get(&sibling))?;
            let subnet = get_subnet_for_canister(env.get_agent(), next_to).await?;
            Ok(Some(SubnetSelection::Subnet { subnet }))
        } else {
            Ok(self
                .subnet_type
                .map(|subnet_type| {
                    SubnetSelection::Filter(SubnetFilter {
                        subnet_type: Some(subnet_type),
                    })
                })
                .or_else(|| self.subnet.map(|subnet| SubnetSelection::Subnet { subnet })))
        }
    }
}
