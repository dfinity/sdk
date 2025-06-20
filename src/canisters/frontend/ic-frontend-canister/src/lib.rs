use candid::ser::IDLBuilder;
use ic_cdk::api::stable;
use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_certified_assets::types::AssetCanisterArgs;

#[init]
fn init(args: Option<AssetCanisterArgs>) {
    ic_certified_assets::init(args);
}

#[pre_upgrade]
fn pre_upgrade() {
    let stable_state = ic_certified_assets::pre_upgrade();
    let value_serializer_estimate = stable_state.estimate_size();
    stable_save_with_capacity((stable_state,), value_serializer_estimate)
        .expect("failed to save stable state");
}

// this is the same as ic_cdk::storage::stable_save,
// but reserves the capacity for the value serializer
fn stable_save_with_capacity<T>(t: T, value_capacity: usize) -> Result<(), candid::Error>
where
    T: candid::utils::ArgumentEncoder,
{
    let mut ser = IDLBuilder::new();
    ser.try_reserve_value_serializer_capacity(value_capacity)?;
    t.encode(&mut ser)?;
    ser.serialize(stable::StableWriter::default())
}

#[post_upgrade]
fn post_upgrade(args: Option<AssetCanisterArgs>) {
    let (stable_state,): (ic_certified_assets::StableState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore stable state");
    ic_certified_assets::post_upgrade(stable_state, args);
}
