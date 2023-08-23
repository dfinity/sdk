use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertMnemonicToKeyError {
    #[error("Failed to derive extended secret key from path: {0}")]
    DeriveExtendedKeyFromPathFailed(bip32::Error),
}
