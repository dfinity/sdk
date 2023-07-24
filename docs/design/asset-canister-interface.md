# Asset Canister Interface

## Introduction

The asset canister interface provides for storage and retrieval of static assets, such as HTML, CSS, JavaScript, images, and other media files. It can store different content encodings of an asset's contents, such as `identity` and `gzip`.

A canister that implements this interface can also return dynamic results from the [http_request](#method-http_request) method.

This document is meant to describe the interface with enough detail to aid in understanding how the asset canister works and in interacting with the asset canister at the code level.  It does not describe the interface in sufficient detail to rise to the level of a specification.

This document describes an interface, not an implementation. The IC SDK bundles one such possible implementation, the [IC Frontend Canister](https://github.com/dfinity/sdk/tree/master/src/canisters/frontend/ic-frontend-canister).

For brevity, this document does not reproduce the candid signatures for every method. 

## Table of Contents

| Section                                                  |
|----------------------------------------------------------|
| [Retrieving Assets](#retrieving-assets)                  |
| [Storing Assets](#storing-assets)                        |
| [Type Reference](#type-reference)                        |
| [Method Reference](#method-reference)                    |
| [Batch Operation Reference](#batch-operation-reference)  |
| [Configuration Reference](#configuration-reference)      |
| [Permission Reference](#permission-reference)            |

## Retrieving Assets

Asset retrieval begins with a call to either [get()](#method-get) or [http_request()](#method-http_request).

### Asset Lookup

The asset canister looks up assets by [key](#key).  An asset key is a unique, case-sensitive identifier.  Asset keys are usually pathnames, and by convention begin with a forward slash.

Examples:
- `/index.html`
- `/img/how-it-works/chain-key-signature.jpg`

#### Aliasing

If no asset with the requested key exists, the asset canister will look for an asset with a different key, where the alternate asset has the [enable_aliasing](#enable-aliasing) field set to `true`.

The aliasing rules are as follows:

- an attempt to retrieve `{some key}/` can instead retrieve `{some key}/index.html`
- an attempt to retrieve `{some key}`, where `{some key}` does not end with `.html`, can instead retrieve either `{some key}.html` or `{some key}/index.html`

Examples:
- an attempt to retrieve `/` can instead retrieve `/index.html`
- an attempt to retrieve `/docs/language-guide/about-this-guide/` can instead retrieve `/docs/language-guide/about-this-guide/index.html`
- an attempt to retrieve `/docs/language-guide/about-this-guide` can instead retrieve `/docs/language-guide/about-this-guide/index.html` or `/docs/language-guide/about-this-guide.html`

### Content Encoding Selection

When retrieving an asset, the caller specifies a list of acceptable [content encodings](#content_encoding). The asset canister will select the first suitable[^1] content encoding from this list.

### Large Assets

While the size of any given asset content encoding is limited only by the canister's available memory, the amount of data that can be passed or returned in a single method call is limited. For this reason, the interface provides for data upload and retrieval in smaller pieces, called "chunks".

The size of each chunk is limited by the message ingress limit.

## Storing Assets

### Batch Updates

The usual method of updating data in the asset canister is by calling the following methods:
1. [create_batch()](#method-create_batch) once.
2. [create_chunk()](#method-create_chunk) one or more times, which can occur concurrently.
3. [commit_batch()](#method-commit_batch) zero or more times with `batch_id: 0`.
4. [commit_batch()](#method-commit_batch) once with the batch ID from step 1, which indicates the batch is complete.

The reason for multiple rather than single calls to [commit_batch][#method-commit_batch] is that certificate computation for an entire batch may exceed per-message computation limits.

### Batch Updates By Proposal

If a [Service Nervous System](https://internetcomputer.org/docs/current/developer-docs/integrations/sns/)  (SNS) controls an asset canister, it can update the assets by proposal. In this scenario, there are two principals:
- The Preparer, which must have the [Prepare](#permission-prepare) permission. This principal prepares the proposal by uploading data and proposing changes to be committed.
- The Committer, which must have the [Commit](#permission-commit) permission. This principal commits the previously-proposed changes.

In this scenario, the Preparer calls the following methods:
1. [create_batch()](#method-create_batch) once.
2. [create_chunk()](#method-create_chunk) one or more times, which can occur concurrently.
3. [propose_commit_batch()](#method-propose_commit_batch) once.
4. [compute_evidence()](#method-compute_evidence) until the method returns `Some(evidence)`.

The Preparer then furnishes the Committer with the following information:
- the batch ID
- the computed evidence

The Committer then calls the following method upon approval of the proposal:
1. [commit_proposed_batch()](#method-commit_proposed_batch)

If the proposal is not approved, the Preparer must call [delete_batch()](#method-delete_batch).  Until this is done, all calls to [create_batch()](#method-create_batch) will fail.

### Individual Updates

It is also possible to upload a single content encoding of a single asset by calling the [store()](#method-store) method.  The size of the content encoding must not exceed the message ingress limit.

## Type Reference

### Asset

#### Key

The `key` is a case-sensitive string that identifies the asset. By convention, all asset keys begin with a forward slash. For example, `/index.html`.

#### Content Type

The `content_type` field is a string that identifies the type of the asset, such as `text/plain` or `image/jpeg`. It is used to set the `Content-Type` header when serving the asset over HTTP.

#### Content Encodings

The asset canister can store and serve multiple encodings of the same asset. Each encoding is identified by a `content_encoding` string, such as `identity` or `gzip`. It is used to set the `Content-Encoding` header when serving the asset over HTTP.

The `identity` encoding corresponds to the original, unencoded asset contents.

#### Content Chunks

Each encoding contains one or more "chunks" of data. The size of each chunk is limited by the message ingress limit.

Content chunks can have any size that fits within the message ingress limit, but for a given asset encoding, all chunks except the last must have the same size.

#### Content Hash

The `sha256` field contains the SHA-256 hash of the entire asset encoding. It is used to set the `ETag` header when serving the asset over HTTP, and to ensure coherence when retrieving a content encoding with more than one query call.

#### Max Age

The `max_age` field is the maximum number of seconds that the asset can be cached by a browser or CDN. It is used to set the `max-age` value of the `Cache-Control` header when serving the asset over HTTP.

#### Headers

The `headers` field is a list of additional headers to set when serving the asset over HTTP.

#### Enable Aliasing

This field enables retrieval of this asset by a different key, according to the [aliasing](#aliasing) rules.

> **NOTE** The interface uses more than one name for this field:
> - `enable_aliasing` in [CreateAsset](#operation-createasset) arguments
> - `is_aliased` in [SetAssetProperties](#operation-setassetproperties) arguments and in the return value of [get_asset_properties()](#method-get_asset_properties).
>
> In all cases, it indicates that the asset's key _might be_ an alias for another asset, not that it is _definitely the case_ for the asset in question.  It will often be `true` for assets which are not an alias for another asset.

#### Raw Access

The `allow_raw_access` field controls whether an asset can be retrieved from `raw.ic0.app` or `raw.icp0.io`. If false (which is the default), then the asset canister will redirect any such attempts to the non-raw URL.

### Batch

The asset canister holds related changes in a batch before committing those changes to assets in its state. The asset canister must retain all data in a batch for at least the [Minimum Batch Retention Duration](#constant-minimum-batch-retention-duration) after creation of the batch itself or creation of any chunk in the batch. 

### Chunk

A chunk is sequence of bytes comprising all or part of a content encoding for an asset.

The size of any chunk cannot exceed the message ingress limit.

## Method Reference

### Method: `get`

```candid
  get: (record {
    key: Key;
    accept_encodings: vec text;
  }) -> (record {
    content: blob; // may be the entirety of the content, or just chunk index 0
    content_type: text;
    content_encoding: text;
    sha256: opt blob; // sha256 of entire asset encoding
    total_length: nat; // all chunks except last have size == content.size()
  }) query;
```

This method looks up the asset with the given key, using [aliasing](#aliasing) rules if the key is not found.

Then, it searches the asset's [content encodings](#content-encodings) in the order specified in `accept_encodings`.  If none are found, it returns an error.  A typical value for `accept_encodings` would be `["gzip", "identity"]`.

Finally, it returns the first chunk of the content encoding.

If `total_length` exceeds the length of the returned `content` blob, this means that there is more than one chunk.  The caller can then call [get_chunk()](#method-get_chunk) to retrieve the remaining chunks.  Note that since all chunks except the last have the same length as the first chunk, the caller can determine the number of chunks by dividing `total_length` by the length of the first chunk.

The `sha256` field is `opt` only because it was added after the initial release of the asset canister.  It must always be present in the response.

### Method: `get_chunk`

```candid
  get_chunk: (record {
    key: Key;
    content_encoding: text;
    index: nat;
    sha256: opt blob;
  }) -> (record { content: blob }) query;
```

This method looks up the asset with the given key, using [aliasing](#aliasing) rules if the key is not found.

It returns the chunk with the given index of the specified content encoding of the asset.

The asset canister returns an error if the `sha256` field is not present, or if the `sha256` field does not match the hash of the content encoding.  This protects against changes to the content encoding in between calls to [get()](#method-get) and [get_chunk()](#method-get_chunk).

### Method: `list`

```candid
  list : (record {}) -> (vec record {
    key: Key;
    content_type: text;
    encodings: vec record {
      content_encoding: text;
      sha256: opt blob; // sha256 of entire asset encoding
      length: nat;
      modified: Time;
    };
  }) query;
```

This method returns a list of all assets.

The `sha256` field is `opt` only because it was added after the initial release of the asset canister.  It must always be present in the response.

### Method: `http_request`

This method returns an HTTP response for the given HTTP request.

### Method: `http_request_streaming_callback`

If the response to an `http_request` call includes a `streaming_strategy`, then this will be the value of the `callback`.

### Method: `create_batch`

This method creates a new [batch](#batch) and returns its ID.

Preconditions:
- No batch exists for which [propose_commit_batch()](#method-propose_commit_batch) has been called.
- Creation of a new batch would not exceed batch creation limits.

Required Permission: [Prepare](#permission-prepare)

### Method: `create_chunk`

```candid
  create_chunk: (
    record { 
      batch_id: BatchId;
      content: blob 
    }
  ) -> (record { 
    chunk_id: ChunkId
  });
```

This method stores a content chunk and extends the batch expiry time.

When creating chunks for a given content encoding, the size of each chunk except the last must be the same.

The asset canister must retain all data related to a batch for at least the [Minimum Batch Retention Duration](#constant-minimum-batch-retention-duration) after creating a chunk in a batch.

Preconditions:
- The batch exists.
- Creation of the chunk would not exceed chunk creation limits.

Required Permission: [Prepare](#permission-prepare)

### Method: `commit_batch`

```candid
  commit_batch: (record {
    batch_id: BatchId;
    operations: vec BatchOperationKind
  }) -> ();
```

The `commit_batch` method executes the specified batch operations in the order listed. The method traps if there is an error executing any operation, so either all or none of the operations will be applied.

After executing the operations, this method deletes the batch associated with `batch_id`. It is valid to pass `0` for batch_id, in which case this method does not delete any batch. This allows multiple calls to `commit_batch` to execute operations from a large batch, such that no call to `commit_batch` exceeds per-call computation limits.  The final call to `commit_batch` should include the batch ID, in order to delete the batch.

| Operation                                           | Description                           |
|-----------------------------------------------------|---------------------------------------|
| [CreateAsset](#operation-createasset)               | Creates a new asset.                  |
| [SetAssetContent](#operation-setassetcontent)       | Adds or changes content for an asset. |
| [SetAssetProperties](#operation-setassetproperties) | Changes properties for an asset.      |
| [UnsetAssetContent](#operation-unsetassetcontent)   | Removes content for an asset.         |
| [DeleteAsset](#operation-deleteasset)               | Deletes an asset.                     |
| [Clear](#operation-clear)                           | Deletes all assets.                   |

Required Permission: [Commit](#permission-commit)

### Method: `delete_batch`

The `delete_batch` method deletes a single batch and any related chunks.

Required Permission: [Prepare](#permission-prepare)

### Method: `propose_commit_batch`

This method takes the same arguments as `commit_batch`, but does not execute the operations. Instead, it stores the operations in a "proposed batch" for later execution by the `commit_proposed_batch` method.

Required permission: [Prepare](#permission-prepare)

### Method: `compute_evidence`

The `compute_evidence` method computes a hash over the proposed commit batch arguments.

Since calculation of this hash may exceed per-message computation limits, this method computes the hash iteratively, saving its work as it goes. Once it completes the computation, it saves the hash as `evidence` to be checked later.

The method will return `None` if the hash computation has not yet completed, or `Some(evidence)` if the hash computation has been completed.

The returned `evidence` value must be passed to the `commit_proposed_batch` method.

After the hash computation has completed, the batch will no longer expire. The batch will remain until one of the following occurs:
- a call to [commit_proposed_batch()]
- a call to [delete_batch()]
- the canister is upgraded

Required permission: [Prepare](#permission-prepare)

### Method: `commit_proposed_batch`

This method executes the operations previously supplied by [propose_commit_batch()](#method-propose_commit_batch), and deletes the batch.

Preconditions:
- The batch exists.
- The batch has proposed commit batch arguments.
- Evidence computation has completed.
- Evidence passed in the arguments matches the evidence previously computed by [compute_evidence()](#method-compute_evidence).

Required permission: [Commit](#permission-commit)

### Method: `grant_permission`

This method grants a permission to a principal.

Callable by: Principals with [ManagePermissions](#permission-managepermissions) permission, and canister controllers.

### Method: `revoke_permission`

This method revokes a permission from a principal.

Callable by: Principals with [ManagePermissions](#permission-managepermissions) permission, and canister controllers. Also, any principal can revoke any of its own permissions.

### Method: `list_permitted`

This method returns a list of principals that have the given permission.

### Method: `authorize`

> **NOTE**: This method is deprecated. Use [grant_permission()](#method-grant_permission) instead.

This method grants the [Commit](#Permission_commit) permission to a principal.

Callable by: Principals with [ManagePermissions](#permission-managepermissions) permission, and canister controllers.

### Method: `deauthorize`

> **NOTE**: This method is deprecated. Use [revoke_permission()](#method-revoke_permission) instead.

This method revokes the [Commit](#Permission_commit) permission from a principal.

Callable by: Principals with [ManagePermissions](#permission-managepermissions) permission, and canister controllers. Also, any principal can revoke anny of its own permissions.

### Method: `list_authorized`

> **NOTE**: This method is deprecated. Use [list_permitted()](#method-list_permitted) instead.

This method returns a list of principals that have the [Commit](#Permission_commit) permission.

### Method: `configure`

This method configures the asset canister. See [Configuration Reference](#configuration-reference) for details.

### Method: `get_configuration`

This method returns the configuration of the asset canister.

### Method: `get_asset_properties`

This method returns the properties of the asset with the given key.

### Method: `certified_tree`

This method returns the certified tree.

### Convenience Methods

Each of these methods is the equivalent of its respective batch operation.

> **NOTE**: While they are provided for "convenience," some don't actually make sense.  For example, [set_asset_content()](#operation-setassetcontent) requires chunk ids, but [create_chunk()](#method-createchunk) requires a batch.
>
> These methods may be deprecated in the future.  It is recommended to instead call [commit_batch()](#method-commitbatch) with a single operation, specifying batch ID 0.

| Method                   | Operation                                           |
|--------------------------|-----------------------------------------------------|
| `create_asset()`         | [CreateAsset](#operation-createasset)               |
| `delete_asset()`         | [DeleteAsset](#operation-deleteasset)               |
| `set_asset_content()`    | [SetAssetContent](#operation-setassetcontent)       |
| `set_asset_properties()` | [SetAssetProperties](#operation-setassetproperties) |
| `unset_asset_content()`  | [UnsetAssetContent](#operation-unsetassetcontent)   |
| `clear()`                | [Clear](#operation-clear)                           |

Each of these methods requires permission: [Commit](#permission-commit)

### Validation Methods

These methods validate the arguments for the corresponding methods.  They are required for the SNS to be able to call the corresponding methods.

- `validate_grant_permission()`
- `validate_revoke_permission()`
- `validate_take_ownership()`
- `validate_commit_proposed_batch()`
- `validate_configure()`

## Batch Operation Reference

### Operation: `CreateAsset`

```candid
type CreateAssetArguments = record {
  key: Key;
  content_type: text;
  max_age: opt nat64;
  headers: opt vec HeaderField;
  enable_aliasing: opt bool;
  allow_raw_access: opt bool;
};
```

This operation creates a new asset.  An asset with the given key must not already exist.

### Operation: `SetAssetContent`

```candid
type SetAssetContentArguments = record {
  key: Key;
  content_encoding: text;
  chunk_ids: vec ChunkId;
  sha256: opt blob;
};
```

This operation adds or changes a single content encoding for an asset.  It also updates the modification time of the content encoding.

If `sha256` is not passed, the asset canister will compute the hash of the content.

### Operation: `SetAssetProperties`

```candid
type SetAssetPropertiesArguments = record {
  key: Key;
  max_age: opt opt nat64;
  headers: opt opt vec HeaderField;
  allow_raw_access: opt opt bool;
  is_aliased: opt opt bool;
};
```

This operation sets some or all properties of an asset.

### Operation: `UnsetAssetContent`  

```candid
type UnsetAssetContentArguments = record {
  key: Key;
  content_encoding: text;
};
```

This operation removes a single content encoding for an asset.

### Operation: `DeleteAsset`    

```candid
type DeleteAssetArguments = record {
  key: Key;
};
```

This operation deletes a single asset.

### Operation: `Clear`

```candid
type ClearArguments = record {};
```

This operation deletes all assets.

## Configuration Reference

These are set by the [configure()](#method-configure) method.  All limits default to unlimited.

| Configuration | Description                                                                         |
|---------------|-------------------------------------------------------------------------------------|
| `max_batches` | The maximum number of batches being uploaded at one time.                           |
| `max_chunks`  | The maximum number of chunks across all batches being uploaded.                     |
| `max_bytes`   | The maximum number of total size of content bytes across all chunks being uploaded. |

## API Versions

### API Version 1

This version added `SetAssetProperties` to `BatchOperationKind`.

## Permissions

### Permission: `Commit`

Permits changes to the assets served by the asset canister.

Any principal with this permission can also call any method callable with the [Prepare](#permission-prepare) permission.

### Permission: `Prepare`

Permits upload of data to the canister to be committed later by a principal with the [Commit](#permission-commit) permission.

### Permission: `ManagePermissions`

Permits a principal to grant and revoke permissions to other principals.

## Constants

### Constant: Minimum Batch Retention Duration

The asset canister must retain all data related to a batch for at least 5 minutes after creating that batch or creating a chunk within it.

[^1] This term is intentionally vague.

