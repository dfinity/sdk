pub trait Retryable {
    fn is_retryable(&self) -> bool;
}
