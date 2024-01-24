const REPLICA_REV_STR: &str = env!("DFX_ASSET_REPLICA_REV");

pub fn replica_rev() -> &'static str {
    REPLICA_REV_STR
}
