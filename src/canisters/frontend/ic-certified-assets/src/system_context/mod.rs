pub mod canister_env;

use std::cell::{Ref, RefCell};

use canister_env::CanisterEnv;
use ic_cdk::api::time;

/// Context that is available only inside canister runtime.
pub struct SystemContext {
    canister_env: RefCell<Option<CanisterEnv>>,
    pub current_timestamp_ns: u64,
}

impl SystemContext {
    pub fn new() -> Self {
        Self {
            // We do not load the canister env here, because it might not be needed.
            // Users should call [Self::get_canister_env] to get the canister env,
            // which takes care of loading the canister env if it is not already loaded.
            canister_env: RefCell::new(None),
            current_timestamp_ns: time(),
        }
    }

    #[cfg(test)]
    pub fn new_with_options(canister_env: Option<CanisterEnv>, current_timestamp_ns: u64) -> Self {
        Self {
            canister_env: RefCell::new(canister_env),
            current_timestamp_ns,
        }
    }

    /// Returns the current canister environment, loading it if it is not already loaded.
    pub fn get_canister_env(&self) -> Ref<'_, CanisterEnv> {
        if self.canister_env.borrow().is_none() {
            let canister_env = CanisterEnv::load();
            self.canister_env.borrow_mut().replace(canister_env);
        }
        Ref::map(self.canister_env.borrow(), |opt| {
            opt.as_ref().expect("CanisterEnv should be initialized")
        })
    }

    pub fn instruction_counter(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            ic_cdk::api::performance_counter(0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // For tests/non-wasm, return 0 or a mock value if needed.
            // Since we don't have a mock setup here yet, 0 is safe as it won't trigger limits.
            0
        }
    }
}

impl Default for SystemContext {
    fn default() -> Self {
        Self::new()
    }
}
