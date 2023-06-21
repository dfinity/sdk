# What this asset contains
This asset is here to build a motoko playground locally so that we can test playground interactions without relying on mainnet.
This asset only contains the parts that are necessary to test our integration with it. The other parts (e.g. the frontend) are stripped out to have quicker deployments.
Included are:
- The `wasm-utils` canister, which performs instrumentation on the deployed wasm (inject profiling, strip forbidden function calls like `add_cycles`).
- The `backend` canister, which manages the playground canisters and lends them out to users.

# How to update the `playground_backend` asset
- Go to https://github.com/dfinity/motoko-playground
- Updating the `backend` canister
    - replace `service/pool` with the live version
    - replace `mops.toml` with the live version
- Updating the `wasm-utils` canister
    - We don't build this canister here because compiling it every time in CI takes far too long.
    - Navigate to `service/wasm-utils`
    - Run `./build.sh`
    - Replace `wasm-utils.wasm` in this asset with `service/wasm-utils/target/wasm32-unknown-unknown/release/wasm-utils.wasm`
    - Replace `wasm-utils.did` in this asset with `service/wasm-utils/wasm-utils.did`