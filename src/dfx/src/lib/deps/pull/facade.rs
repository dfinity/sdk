use candid::Principal;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref FACADE: HashMap<Principal, Vec<Principal>> = {
        let mut m = HashMap::new();
        // ICP ledger
        m.insert(Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(), vec![]);
        m
    };
}

pub(super) fn facade_dependencies(canister_id: &Principal) -> Option<Vec<Principal>> {
    FACADE.get(canister_id).cloned()
}
