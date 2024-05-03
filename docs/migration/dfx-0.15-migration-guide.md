# dfx 0.15.0 Migration Guide

## Restrict new identity names to safe characters

New identities like `dfx identity new my/identity` or `dfx identity new 'my identity'` can easily lead to problems, either for dfx internals or for usability.
New identities are now restricted to the characters `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.-_@0123456789`.
Existing identities are not affected by this change.

## Use `dfx start` rather than `dfx bootstrap` and `dfx replica`

If you have been using `dfx bootstrap` and `dfx replica`, use `dfx start` instead.  If you have a good reason why we should keep these commands, please contribute to the discussion at https://github.com/dfinity/sdk/discussions/3163

## Install nns and sns extensions if using this functionality

We've removed the `dfx nns` and `dfx sns` commands from dfx. Both have now been turned into dfx extensions. In order to obtain them, please run `dfx extension install nns` and `dfx extension install sns` respectively. After the installation, you can use them as you did before: `dfx nns ...`, and `dfx sns ...`.

## frontend canister: Delete assets before re-creating them

You don't have to change anything if you are using `dfx deploy`, icx-asset sync`, or the ic-asset crate `sync` methods.  All of these already take care of this, in current and previous versions.

The CreateAsset operation and create_asset method now fail if the asset already exists.  Previously, these were no-op if the content type matched.

For an asset that already exists:
- before adding a `CreateAssetOperation`, add a `DeleteAssetOperation`
- before calling `create_asset()`, call `delete_asset()`

## frontend canister: Always pass Some(sha256) parameter to `http_request_streaming_callback` and `get_chunk`

For http_request and http_request_streaming_callback, there should be no change: all callers of http_request_streaming_callback should be passing the entire token returned by http_request, which includes the sha256 parameter.

Any callers of `get_chunk` should make sure to always pass the `sha256` value returned by the `get` method.  It will always be present.
