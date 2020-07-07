pub(crate) mod management_canister;

pub(crate) mod public {
    use super::*;

    pub use management_canister::{InstallMode, ManagementCanister};
}
