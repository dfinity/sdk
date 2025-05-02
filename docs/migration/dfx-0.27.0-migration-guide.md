# dfx 0.27.0 migration guide

## Removal of the replica

dfx 0.27.0 no longer bundles the replica and the `--replica` flag to `dfx start` produces an error. The `--pocketic` flag will remain, but does nothing. This means that if you were depending on any replica-specific implementation details, you will need to adapt your project to the PocketIC equivalent.

### If you have been using an agent to create canisters

Creating a canister via `CreateCanisterBuilder::as_provisional_create_with_*`/`aaaaa-aa.provisional_create_canister_with_cycles` requires that the 'effective canister ID' be specified with a canister on the same subnet that the canister you are creating will go on. When using the replica, `ic-utils` was able to fill this in for you. When using PocketIC, you must specify it yourself. You can get the current default effective canister ID from dfx via `dfx info default-effective-canister-id`. You may alternatively process the topology at `http://localhost:{port}/_/topology`, which will be a JSON object containing at least the path `.default_effective_canister_id.canister_id`, which will contain the base64 encoding of the principal's byte representation.

### If you have been referring to the adapter or sandbox processes in scripts

PocketIC does not use the adapter processes or sandbox processes, and dfx no longer bundles them. You should remove references to these processes from your scripts.

## Removal of other tools

dfx 0.27.0 no longer bundles the `ic-admin`, `sns`, or `ic-nns-init` CLI tools. If you are using these, you can download them from the following URL: `https://download.dfinity.systems/ic/<rev>/binaries/x86_64-linux/<bin>.gz`, where `<rev>` is the git commit hash of an elected replica release and `<bin>` is `ic-admin`, `ic-nns-init`, or `sns`.
 