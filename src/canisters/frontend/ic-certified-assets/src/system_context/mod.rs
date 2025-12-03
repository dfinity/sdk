pub mod canister_env;

use std::cell::{Ref, RefCell};

use canister_env::CanisterEnv;
use ic_cdk::api::time;

/// Context that is available only inside canister runtime.
///
/// # Example
///
/// ```
/// use ic_certified_assets::system_context::SystemContext;
/// use ic_certified_assets::with_state_mut;
/// use ic_certified_assets::types::CommitBatchArguments;
/// use ic_cdk::api::{certified_data_set, trap};
/// use ic_cdk::update;
///
/// #[update]
/// pub fn commit_batch(arg: CommitBatchArguments) {
///     let system_context = SystemContext::new();
///
///     with_state_mut(|s| {
///         if let Err(msg) = s.commit_batch(arg, &system_context) {
///             trap(&msg);
///         }
///         certified_data_set(s.root_hash());
///     });
/// }
/// ```
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
