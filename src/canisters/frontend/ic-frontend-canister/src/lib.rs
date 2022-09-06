use ic_cdk_macros::{init, post_upgrade, pre_upgrade};

#[init]
fn init() {
    ic_certified_assets::init();
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::storage::stable_save((ic_certified_assets::pre_upgrade(),))
        .expect("failed to save stable state");
}

#[post_upgrade]
fn post_upgrade() {
    let (stable_state,): (ic_certified_assets::StableState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore stable state");
    ic_certified_assets::post_upgrade(stable_state);
}
