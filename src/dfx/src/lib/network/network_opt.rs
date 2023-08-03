use clap::{ArgGroup, Args};

#[derive(Args, Clone, Debug, Default)]
#[clap(
group(ArgGroup::new("network-select").multiple(false)),
)]
pub struct NetworkOpt {
    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[arg(long, global(true), group = "network-select")]
    network: Option<String>,

    /// Shorthand for --network=playground.
    /// Borrows short-lived canisters on the real IC network instead of creating normal canisters.
    #[clap(long, global(true), group = "network-select")]
    playground: bool,

    /// Shorthand for --network=ic.
    #[clap(long, global(true), group = "network-select")]
    ic: bool,
}

impl NetworkOpt {
    pub fn to_network_name(&self) -> Option<String> {
        if self.playground {
            Some("playground".to_string())
        } else if self.ic {
            Some("ic".to_string())
        } else {
            self.network.clone()
        }
    }
}
