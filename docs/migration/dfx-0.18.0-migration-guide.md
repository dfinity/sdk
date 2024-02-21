# dfx 0.18.0 Migration Guide

## Use dfxvm rather than `dfx upgrade`

We've removed the `dfx upgrade` command.  Please use the [dfx version manager][dfxvm] to manage dfx versions instead.

[dfxvm]: https://github.com/dfinity/dfxvm

## Removed the deprecated <canister_name>_CANISTER_ID environment variables

If you're still referencing environment variables of the form
`<canister_name>_CANISTER_ID` or `<canister_name_uppercase>_CANISTER_ID`,
change to using `CANISTER_ID_<canister_name_uppercase>` exclusively.
