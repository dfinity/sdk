use ic_utils::Canister;
use std::time::Duration;

pub(crate) struct CanisterCallParams<'a> {
    pub(crate) canister: &'a Canister<'a>,
    pub(crate) timeout: Duration,
}
