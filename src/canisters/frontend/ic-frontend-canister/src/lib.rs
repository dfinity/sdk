use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_certified_assets::types::AssetCanisterArgs;

#[init]
fn init(args: Option<AssetCanisterArgs>) {
    ic_certified_assets::init(args);
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::storage::stable_save((ic_certified_assets::pre_upgrade(),))
        .expect("failed to save stable state");
}

#[post_upgrade]
fn post_upgrade(args: Option<AssetCanisterArgs>) {
    let (stable_state,): (ic_certified_assets::StableState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore stable state");
    ic_certified_assets::post_upgrade(stable_state, args);
}
