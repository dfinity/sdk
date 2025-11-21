use candid::Principal;

pub const MAINNET_BITCOIN_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x01, 0xA0, 0x00, 0x01, 0x01, 0x01]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_canister_id() {
        assert_eq!(
            MAINNET_BITCOIN_CANISTER_ID,
            Principal::from_text("g4xu7-jiaaa-aaaan-aaaaq-cai").unwrap()
        );
    }
}
