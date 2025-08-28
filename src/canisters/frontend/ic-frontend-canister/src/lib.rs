use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk::{println, stable};
use ic_certified_assets::types::AssetCanisterArgs;

#[init]
fn init(args: Option<AssetCanisterArgs>) {
    ic_certified_assets::init(args);
}

#[pre_upgrade]
fn pre_upgrade() {
    println!("pre_upgrade started...");
    let stable_state = ic_certified_assets::pre_upgrade();
    save_stable_state(&stable_state);
    #[cfg(target_arch = "wasm32")]
    println!(
        "pre_upgrade memory size (bytes): {}",
        core::arch::wasm32::memory_size(0) * 65_536
    );
    println!("pre_upgrade completed!");
}

#[post_upgrade]
fn post_upgrade(args: Option<AssetCanisterArgs>) {
    let stable_state = if is_candid_stable_state() {
        // backward compatibility
        println!("restoring candid stable state");
        let (stable_state,) =
            ic_cdk::storage::stable_restore::<(ic_certified_assets::StableState,)>()
                .expect("failed to restore stable state");
        stable_state.into()
    } else {
        println!("restoring serde_cbor stable state");
        load_stable_state().expect("failed to deserialize stable state")
    };
    ic_certified_assets::post_upgrade(stable_state, args);
    println!("post_upgrade completed");
}

fn save_stable_state(stable_state: &ic_certified_assets::StableStateV2) {
    let mut stable_writer = stable::StableWriter::default();
    serde_cbor::to_writer(&mut stable_writer, stable_state)
        .expect("failed to serialize stable state");
}

fn is_candid_stable_state() -> bool {
    let mut maybe_magic_bytes = vec![0u8; 4];
    stable::stable_read(0, &mut maybe_magic_bytes);
    maybe_magic_bytes == b"DIDL"
}

fn load_stable_state() -> Result<ic_certified_assets::StableStateV2, serde_cbor::Error> {
    let stable_reader = stable::StableReader::default();
    from_reader_ignore_trailing_data(stable_reader)
}

fn from_reader_ignore_trailing_data<T, R>(reader: R) -> Result<T, serde_cbor::Error>
where
    T: serde::de::DeserializeOwned,
    R: std::io::Read,
{
    let mut deserializer = serde_cbor::de::Deserializer::from_reader(reader);
    let value = serde::de::Deserialize::deserialize(&mut deserializer)?;
    // we do not call deserializer.end() here
    // because we want to ignore trailing data loaded from stable memory
    Ok(value)
}

ic_certified_assets::export_canister_methods!();
