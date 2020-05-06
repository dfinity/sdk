//! A provider is responsible for signing functionality and generating
//! principals. A user profile may utilize multiple principal providers.

pub mod basic;

pub use basic::BasicProvider;
