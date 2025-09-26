# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING**: Implement `serde::Serialize` and `serde::Deserialize` for stable state structures:
  - Moved all stable state structures to the `stable_machine::v1` module, renaming them to `StableStateV1`, `StableConfigurationV1`, `StableStatePermissionsV1`, `StableAssetV1`, `StableAssetEncodingV1`
  - Removed `StableState` struct
  - Introduced new `state_machine::v2` module with the new `StableStateV2` struct, which implements `serde::Serialize` and `serde::Deserialize`. This allows to serialize and deserialize the state using serde-compatible libraries, such as `serde_cbor`.
  - Added conversion between legacy `StableStateV1` and new `StableStateV2` structs
  - `pre_upgrade()` now returns `StableStateV2` instead of `StableStateV1`
  - `post_upgrade()` now accepts `StableStateV2` parameter instead of `StableStateV1`
  - Removed `estimate_size()` methods from the `StableStateV1`, `StableConfigurationV1`, `StableStatePermissionsV1`, `StableAssetV1`, `StableAssetEncodingV1` structs
- **BREAKING**: Use `BTreeMap` instead of `HashMap` for headers to guarantee deterministic ordering.
  - Changed `StableAssetV2.headers` to use `BTreeMap<String, String>` instead of `HashMap<String, String>`
  - Changed `Asset.headers` to use `BTreeMap<String, String>` instead of `HashMap<String, String>`
  - Changed `CreateAssetArguments.headers` to use `BTreeMap<String, String>` instead of `HashMap<String, String>`
  - Changed `AssetProperties.headers` to use `BTreeMap<String, String>` instead of `HashMap<String, String>`
  - Changed `SetAssetPropertiesArguments.headers` to use `BTreeMap<String, String>` instead of `HashMap<String, String>`
- **BREAKING**: Sets the `ic_env` cookie for html files, which contains the root key and the canister environment variables that are prefixed with `PUBLIC_`. Please note that this version of the `ic-certified-assets` is only compatible with PocketIC **v10** and above.

#### Migration guide

To migrate canisters that use the `ic-certified-assets` library to the new serde-serializable stable state:

1. Upgrade to the latest `ic-certified-assets` which exports `StableStateV2` and implements `serde::{Serialize, Deserialize}` for stable state types.

2. Choose a serde-compatible library to serialize and deserialize the stable state, such as [`serde_cbor`](https://crates.io/crates/serde_cbor), and add it to your canister's dependencies.

3. Update the upgrade hooks to persist the new serialized state in stable memory and keep backward compatibility with existing deployments that stored Candid:
    ```rust
    // In this example, the serde-compatible library of choice is `serde_cbor`.

    use ic_cdk::stable;
    use ic_certified_assets::{StableStateV1, StableStateV2, types::AssetCanisterArgs};

    pub fn save_stable_state(stable_state: &StableStateV2) -> Result<(), serde_cbor::Error> {
        let mut stable_writer = stable::StableWriter::default();
        serde_cbor::to_writer(&mut stable_writer, stable_state)
    }

    pub fn is_candid_stable_state() -> bool {
        let mut maybe_magic_bytes = vec![0u8; 4];
        stable::stable_read(0, &mut maybe_magic_bytes);
        maybe_magic_bytes == b"DIDL"
    }

    pub fn load_candid_stable_state() -> Result<StableStateV1, String> {
        let (stable_state,) = ic_cdk::storage::stable_restore()?;
        Ok(stable_state)
    }

    pub fn load_stable_state() -> Result<StableStateV2, serde_cbor::Error> {
        let stable_reader = stable::StableReader::default();
        from_reader_ignore_trailing_data(stable_reader)
    }

    fn from_reader_ignore_trailing_data<T, R>(reader: R) -> Result<T, serde_cbor::Error>
    where
        T: serde::de::DeserializeOwned,
        R: std::io::Read,
    {
        let mut deserializer = serde_cbor::de::Deserializer::from_reader(reader);
        let value = serde::de::Deserialize::deserialize(&mut deserializer)?;
        // we do not call deserializer.end() here
        // because we want to ignore trailing data loaded from stable memory
        Ok(value)
    }

    #[ic_cdk::pre_upgrade]
    fn pre_upgrade() {
        let stable_state = ic_certified_assets::pre_upgrade();
        save_stable_state(&stable_state).expect("failed to serialize stable state");
    }

    #[ic_cdk::post_upgrade]
    fn post_upgrade(args: Option<AssetCanisterArgs>) {
        let stable_state = if is_candid_stable_state() {
            // backward compatibility
            load_candid_stable_state()
                .expect("failed to restore candid stable state")
                .into()
        } else {
            load_stable_state().expect("failed to deserialize stable state")
        };
        ic_certified_assets::post_upgrade(stable_state, args);
    }
    ```

This way, you maintain backward compatibility with the existing deployment of your asset canister, which was using Candid to save and load the stable state. An implementation reference can be found in the [Asset Canister source code](https://github.com/dfinity/sdk/tree/master/src/canisters/frontend/ic-frontend-canister).

## [0.3.0] - 2025-06-26

### Added

- The stored state can now be directly accessed with `with_state`
- Asset permissions can be set in the initialization parameters when installing or upgrading
- Added bulk operations for chunk uploading and committing
- Added support for response verification v2 certificate expressions
- Added configurable upload limits for chunks and batches
- Added an API for proposing, validating, and committing asset change batches, for use in SNSes
- Added functions for getting and setting asset properties
- Added `list_authorized` and `deauthorize` functions
- Added `take_ownership` function for clearing the ACL
- Added a more fine-grained permission system to the ACL

### Changed

- Exported methods are now declared explicitly with the `export_canister_methods!` macro. The implementations of these methods are now public and can be invoked explicitly
- Converted `list_permitted` to an update call
- Domain redirection now prefers `icp0.io` over `ic0.app`
- Authorized users can no longer authorize other users

## [0.2.5] - 2022-08-22
### Added 
- Support for asset caching based on [ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- Automatic redirection of all traffic from `.raw.ic0.app` domain to `.ic0.app`

## [0.2.4] - 2022-07-12
### Fixed
- headers field in Candid spec accepts mmultiple HTTP headers

## [0.2.3] - 2022-07-06
### Added
- Support for setting custom HTTP headers on asset creation 

## [0.2.2] - 2022-05-12
### Fixed
- Parse and produce ETag headers with quotes around the hash

## [0.2.1] - 2022-05-12
### Fixed
- Make StableState public again

## [0.2.0] - 2022-05-11
### Added
- Support for asset caching based on [ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
- Support for asset caching based on [max-age](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control)
- Automatic redirection of all traffic from `.raw.ic0.app` domain to `.ic0.app`

## [0.1.0] - 2022-02-02
### Added
- First release
