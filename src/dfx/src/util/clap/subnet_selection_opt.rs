use candid::Principal;
use clap::{ArgGroup, Args};

use crate::lib::cycles_ledger_types::create_canister::{SubnetFilter, SubnetSelection};

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
}

impl SubnetSelectionOpt {
    pub fn into_subnet_selection(self) -> Option<SubnetSelection> {
        self.subnet_type
            .map(|subnet_type| {
                SubnetSelection::Filter(SubnetFilter {
                    subnet_type: Some(subnet_type),
                })
            })
            .or_else(|| self.subnet.map(|subnet| SubnetSelection::Subnet { subnet }))
    }
}
