pub(crate) mod agent_error;
pub(crate) mod agent_impl;

pub(crate) mod public {
    use super::*;

    pub use agent_error::*;
    pub use agent_impl::Agent;
}

// Tests
mod agent_test;
