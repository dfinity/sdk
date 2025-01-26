#[used]
#[no_mangle]
pub static LARGE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/garbage.bin"));
