use anyhow::bail;
use candid::Principal;
use clap::{ArgGroup, Args};
use fn_error_context::context;

use crate::lib::{
    cycles_ledger_types::create_canister::{SubnetFilter, SubnetSelection},
    environment::Environment,
    error::DfxResult,
    named_canister::UI_CANISTER,
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
    pub async fn into_subnet_selection_type(
        self,
        env: &dyn Environment,
    ) -> DfxResult<SubnetSelectionType> {
        if let Some(sibling) = self.next_to {
            let next_to = Principal::from_text(&sibling)
                .or_else(|_| env.get_canister_id_store()?.get(&sibling))?;
            let subnet = get_subnet_for_canister(env.get_agent(), next_to).await?;
            Ok(SubnetSelectionType::Explicit {
                user_choice: SubnetSelection::Subnet { subnet },
            })
        } else if let Some(subnet_type) = self.subnet_type {
            Ok(SubnetSelectionType::Explicit {
                user_choice: SubnetSelection::Filter(SubnetFilter {
                    subnet_type: Some(subnet_type),
                }),
            })
        } else if let Some(subnet) = self.subnet {
            Ok(SubnetSelectionType::Explicit {
                user_choice: SubnetSelection::Subnet { subnet },
            })
        } else {
            Ok(SubnetSelectionType::Automatic {
                selected_subnet: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub enum SubnetSelectionType {
    Automatic {
        selected_subnet: Option<SubnetSelection>,
    },
    Explicit {
        user_choice: SubnetSelection,
    },
}

impl Default for SubnetSelectionType {
    fn default() -> Self {
        Self::Automatic {
            selected_subnet: None,
        }
    }
}

impl SubnetSelectionType {
    pub fn get_user_choice(&self) -> Option<SubnetSelection> {
        match self {
            SubnetSelectionType::Explicit { user_choice } => Some(user_choice.clone()),
            _ => None,
        }
    }

    #[context("Failed to figure out subnet to create canister on.")]
    pub async fn resolve(&mut self, env: &dyn Environment) -> DfxResult<Option<SubnetSelection>> {
        if matches!(
            self,
            SubnetSelectionType::Automatic {
                selected_subnet: None
            }
        ) {
            self.resolve_automatic(env).await?;
        }

        match self {
            SubnetSelectionType::Explicit { user_choice } => Ok(Some(user_choice.clone())),
            SubnetSelectionType::Automatic { selected_subnet } => Ok(selected_subnet.clone()),
        }
    }

    pub async fn resolve_automatic(
        &mut self,
        env: &dyn Environment,
    ) -> DfxResult<Option<SubnetSelection>> {
        let canisters = env.get_canister_id_store()?.non_remote_user_canisters();
        let subnets: Vec<_> = futures::future::try_join_all(
            canisters
                .into_iter()
                .filter(|(name, _)| name != UI_CANISTER)
                .map(|(_, canister)| get_subnet_for_canister(env.get_agent(), canister)),
        )
        .await?;

        let mut selected_subnet = None;
        for next_subnet in subnets.into_iter() {
            match selected_subnet {
                None => selected_subnet = Some(next_subnet),
                Some(selected_subnet) => {
                    if selected_subnet == next_subnet {
                        continue;
                    } else {
                        bail!("Cannot automatically decide which subnet to target. Please explicitly specify --subnet or --subnet-type.")
                    }
                }
            }
        }

        match selected_subnet {
            None => Ok(None),
            Some(subnet) => {
                let selection = SubnetSelection::Subnet { subnet };
                *self = Self::Automatic {
                    selected_subnet: Some(selection.clone()),
                };
                Ok(Some(selection))
            }
        }
    }
}
