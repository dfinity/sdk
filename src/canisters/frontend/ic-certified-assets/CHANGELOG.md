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
