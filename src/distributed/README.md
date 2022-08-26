
# Building the Wallet Canister

The `wallet.wasm` and `wallet.did` files here are built using `dfx build` in the
https://github.com/dfinity/rust-wallet repo.

To build, clone that repo, run `dfx build`, then copy the `wallet.wasm`.  Or, run
./update-wallet from this directory.  For either, in order for `dfx build` to work,
you will first have to create the canister ids with `dfx start --background` and
`dfx canister create wallet`.

An issue was created to automate this using nix; https://github.com/dfinity-lab/sdk/issues/1078

# Building the Asset Canister

The `assetstorage.wasm.gz` and `assetstorage.did` files here are built using scripts/update-frontend-canister.sh in this repo.

To build, clone that repo, run `dfx build`, then copy the `.dfx/local/canisters/certified_assets/certified_assets.wasm`.

An issue was created to automate this using nix; https://github.com/dfinity-lab/sdk/issues/1078
