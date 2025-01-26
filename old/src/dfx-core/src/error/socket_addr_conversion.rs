use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocketAddrConversionError {
    #[error("Did not find any socket addresses in string '{0}'")]
    EmptyIterator(String),

    #[error("Failed to convert {0} to a socket address")]
    ParseSocketAddrFailed(String, #[source] std::io::Error),
}
