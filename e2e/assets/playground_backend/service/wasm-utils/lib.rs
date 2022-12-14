use candid::{CandidType, Deserialize};
use serde_bytes::ByteBuf;

use ic_wasm::*;

#[derive(CandidType, Deserialize)]
struct Config {
    profiling: bool,
    remove_cycles_add: bool,
    limit_stable_memory_page: Option<u32>,
    backend_canister_id: Option<candid::Principal>,
}

#[ic_cdk_macros::query]
fn transform(wasm: ByteBuf, config: Config) -> ByteBuf {
    let mut m = walrus::Module::from_buffer(&wasm).unwrap();
    if config.profiling {
        instrumentation::instrument(&mut m);
    }
    let resource_config = limit_resource::Config {
        remove_cycles_add: config.remove_cycles_add,
        limit_stable_memory_page: config.limit_stable_memory_page,
        playground_canister_id: config.backend_canister_id,
    };
    limit_resource::limit_resource(&mut m, &resource_config);
    let wasm = m.emit_wasm();
    ByteBuf::from(wasm)
}
