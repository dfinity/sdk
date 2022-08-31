use lazy_static::lazy_static;
lazy_static! {
    static ref REPLICA_REV_STR: String = env!("DFX_ASSET_REPLICA_REV").to_string();
}

pub fn replica_rev() -> &'static str {
    &REPLICA_REV_STR
}
