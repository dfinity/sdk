# dfx 0.18.0 Migration Guide

## Removed the deprecated <canister_name>_CANISTER_ID environment variables

If you're still referencing environment variables of the form
`<canister_name>_CANISTER_ID` or `<canister_name_uppercase>_CANISTER_ID`,
change to using `CANISTER_ID_<canister_name_uppercase>` exclusively.


