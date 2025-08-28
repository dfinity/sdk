use ic_cdk::stable;
use ic_certified_assets::{StableState, StableStateV2};

pub fn save_stable_state(stable_state: &StableStateV2) -> Result<(), serde_cbor::Error> {
    let mut stable_writer = stable::StableWriter::default();
    serde_cbor::to_writer(&mut stable_writer, stable_state)
}

pub fn is_candid_stable_state() -> bool {
    let mut maybe_magic_bytes = vec![0u8; 4];
    stable::stable_read(0, &mut maybe_magic_bytes);
    maybe_magic_bytes == b"DIDL"
}

pub fn load_candid_stable_state() -> Result<StableState, String> {
    let (stable_state,) = ic_cdk::storage::stable_restore()?;
    Ok(stable_state)
}

pub fn load_stable_state() -> Result<StableStateV2, serde_cbor::Error> {
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
