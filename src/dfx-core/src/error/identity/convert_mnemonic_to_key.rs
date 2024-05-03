use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertMnemonicToKeyError {
    #[error("Failed to derive extended secret key from path")]
    DeriveExtendedKeyFromPathFailed(#[source] bip32::Error),
}
