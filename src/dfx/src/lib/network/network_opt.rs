use clap::Args;

#[derive(Args, Clone, Debug, Default)]
pub struct NetworkOpt {
    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[arg(long, global = true, conflicts_with("playground"))]
    network: Option<String>,

    /// Shorthand for --network=playground.
    /// Borrows short-lived canisters on the real IC network instead of creating normal canisters.
    #[clap(long, global(true), conflicts_with("network"))]
    playground: bool,
}

impl NetworkOpt {
    pub fn to_network_name(self) -> Option<String> {
        if self.playground {
            Some("playground".to_string())
        } else {
            self.network
        }
    }
}
