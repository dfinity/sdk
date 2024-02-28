# dfx 0.18.0 Migration Guide

## Use dfxvm rather than `dfx upgrade`

We've removed the `dfx upgrade` command.  Please use the [dfx version manager][dfxvm] to manage dfx versions instead.

[dfxvm]: https://github.com/dfinity/dfxvm

## Use standard canister ID and candid path environment variable names

This version no longer provides environment variable name formats
that we deprecated in dfx 0.14.0.

If you are using environment variables to reference canister IDs and candid paths,
you may need to update your environment variable names.

The only variable names now provided are the following,
all uppercase, with any '-' replaced by '_':
- `CANISTER_CANDID_PATH_<CANISTER_NAME>`
- `CANISTER_ID_<CANISTER_NAME>`

Rust canisters may need to upgrade their dependencies to at least:

```toml
candid = "0.10"
ic-cdk = "0.12"
```
