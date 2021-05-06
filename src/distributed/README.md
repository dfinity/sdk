
# Building the Wallet Canister

The `wallet.wasm` and `wallet.did` files here are built using `dfx build` in the
https://github.com/dfinity/rust-wallet repo.

To build, clone that repo, run `dfx build`, then copy the `wallet.wasm`.

An issue was created to automate this using nix; https://github.com/dfinity-lab/sdk/issues/1078

# Building the Asset Canister

The `assetstorage.wasm` and `assetstorage.did` files here are built using `dfx build` in the
https://github.com/dfinity/certified-assets repo.

To build, clone that repo, run `dfx build`, then copy the `assetstorage.wasm`.

An issue was created to automate this using nix; https://github.com/dfinity-lab/sdk/issues/1078
