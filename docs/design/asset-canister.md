# Overview

Rework the asset canister so that it can store assets that are larger
than the message ingress limit.

Also, support [Http Canister
Queries](https://www.notion.so/Design-HTTP-Canisters-Queries-d6bc980830a947a88bf9148a25169613)
by associating additional metadata with each asset, and content
encodings.

# Background

The asset storage canister at present is a rudimentary key/value store
of static blob data.

The update call requires passing the asset content blob in its entirety,
which limits asset size to the message ingress limit.

## Problem Statement

The main purpose is to make it so asset content length can exceed the
message ingress limit.

## Requirements

-   Can upload assets of any size (within reason, say &lt;2GB)

-   Support upcoming http server

# Prior Art

S3 rest interface

-   <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>

    -   returns Content-Type and so forth along with the body

    -   Note that S3 does not directly support directory structure

# Detailed Design

## Assets

Store per asset:

-   Key

    -   an arbitrary string that identifies the asset

-   Content type

-   Content for one or more content encodings

    -   Content encoding

    -   The actual content (bytes)

    -   A sha256 of the content, calculated by dfx. The asset canister
        treats this as opaque data.

        -   Split up into chunks for retrieval

## Implementation

We will implement the canister in Motoko, because the canister’s
functionality is within the realm of what might be found in a typical
application canister.

## Considered Solutions

### Motoko canister

-   Currently there is not a large body of Motoko libraries for things
    like compression

    -   Mitigated by: compression would probably be better done in dfx
        anyway

### Rust canister with stable memory block

These notes refer to a proposed implementation mapping a FAT32 volume
onto a block of stable memory.

-   Affords access to general Rust libraries

-   Adds complexity in managing data in stable memory

    -   Asset keys are not valid FAT32 paths

    -   Blob storage in files implies management of those files and
        allocation of their filenames

-   Performance considerations:

    -   Retrieval requires reading the content out of stable memory into
        a blob before returning it, since content would not be stored in
        contiguous memory.

    -   Depending on the number of asset content blobs we expect an
        asset canister to hold, it might be necesary to split up the
        files that store content into subdirectories.

#### FAT32 implementation notes (rejected)

Note that at present the Rust CDK does not have libraries for data
structures stored in stable memory.

We would use a [FAT32 library](https://crates.io/crates/fat32) to map a
FAT32 volume onto a block of stable memory in order to store both asset
metadata and content blobs.

Directory structure on this FAT32 filesystem:

    /meta
      hex/digits/of/asset/key.can
    /content
      ${blob_id}.dat

====== The `meta` directory

This directory stores metadata for each asset.

Since keys are of arbitrary length and can contain characters that are
not valid in FAT32 filenames, we will convert the asset key into hex
digits representing the UTF-8 bytes, and split these hex digits into
groups of 8 characters, naming directories and finally a filename.

Each file will store a single Candid record:

    record {
      content_type: text;
      content: vec record {
        content_encoding: text;
        blob_id: text;
      };
    };

====== The `content` directory

This directory stores asset content per content encoding, in individual
files.

At the API level, blob ids are text (a temporary handle), only used
until the blob data is "set" on the asset.

In practice, blob ids will be numbers. To choose them, we can start with
a monotonically increasing value.

====== Blob ids

Another option would be `timestamp/sequence`, assuming only one
`create_blobs` call per canister per block height.

### Rust canister with unstable memory

-   This would be a reasonable implementation and would not require
    changing the interface

-   Would require uploading all assets on every upgrade

    -   Mitigated by: only if any asset changed (not detectable at
        present)

-   Canister-level "upgrade" would only be needed when the asset
    canister wasm changes

    -   It is not obvious how to detect this

## Public API

    type BatchId = nat;
    type ChunkId = nat;
    type Key = text;

    // Create a new asset.  Contents will be attached later with SetContent.
    //   - No-op if asset already exists with the same content type.
    //   - Error if asset already exists with a different content type (delete first).
    type CreateAssetArguments = record {
      key: Key;
      content_type: text;
    };

    // Add or change content for an asset, by content encoding
    type SetAssetContentArguments = record {
      key: Key;
      content_encoding: text;
      chunk_ids: vec ChunkId;
      sha256: opt blob;
    };

    // Remove content for an asset, by content encoding
    type UnsetAssetContentArguments = record {
      key: Key;
      content_encoding: text;
    };

    // Delete an asset
    type DeleteAssetArguments = record {
      key: Key;
    };

    // Future: set up access control
    type SetAssetAclArguments = record {
      key: Key;
      tbd: text;
    };

    // Future: set a time after which to delete an asset
    type SetAssetExpiryArguments = record {
      key: Key;
      tbd: text;
    };

    // Reset everything
    type ClearArguments = record {};

    type BatchOperationKind = variant {
      CreateAsset: CreateAssetArguments;
      SetAssetContent: SetAssetContentArguments;

      UnsetAssetContent: UnsetAssetContentArguments;
      DeleteAsset: DeleteAssetArguments;

      SetAssetAcl: SetAssetAclArguments;
      SetAssetExpiry: SetAssetExpiryArguments;

      Clear: ClearArguments;
    };

    service: {

      get: (record {
        key: Key;
        accept_encodings: vec text;
      }) -> (record {
        content: blob; // may be the entirety of the content, or just chunk index 0
        content_type: text;
        content_encoding: text;
        sha256: opt blob; // sha256 of entire asset encoding, calculated by dfx and passed in SetAssetContentArguments
        total_length: nat; // all chunks except last have size == content.size()
      }) query;

      // if get() returned chunks > 1, call this to retrieve them.
      // chunks may or may not be split up at the same boundaries as presented to create_chunk().
      get_chunk: (record {
        key: Key;
        content_encoding: text;
        index: nat;
        sha256: opt blob;  // sha256 of entire asset encoding, calculated by dfx and passed in SetAssetContentArguments
      }) -> (record { content: blob }) query;

      list: (record {}) -> (vec record {
        key: Key;
        content_type: text;
        encodings: vec record {
          content_encoding: text;
          sha256: opt blob; // sha256 of entire asset encoding, calculated by dfx and passed in SetAssetContentArguments
          length: nat; // Size of this encoding's blob. Calculated when uploading assets.
        };
      }) query;

      create_batch(record {}) -> (record { batch_id: BatchId });

      create_chunk: (record { batch_id: BatchId; content: blob }) -> (record { chunk_id: ChunkId });

      // Perform all operations successfully, or reject
      commit_batch: (record { batch_id: BatchId; operations: vec BatchOperationKind }) -> ();

      create_asset: (CreateAssetArguments) -> ();
      set_asset_content: (SetAssetContentArguments) -> ();
      unset_asset_content: (UnsetAssetContentArguments) -> ();

      delete_asset: (DeleteAssetArguments) -> ();

      set_asset_acl: (SetAssetAclArguments) -> ();
      set_asset_expiry: (SetAssetExpiryArguments) -> ();

      clear: (ClearArguments) -> ();

      // Single call to create an asset with content for a single content encoding that
      // fits within the message ingress limit.
      store: (record {
        key: Key;
        content_type: text;
        content_encoding: text;
        content: blob;
        sha256: opt blob
      }) -> ();
    }

## Security Considerations

For the time being, security controls will continue to be: - assets
writable only by canister owner - assets readable by anyone

## Performance Considerations

The size of the stable memory block in the canister will need to be
roughly double the size required to hold only the assets, because during
upgrades all of the new assets will briefly be stored along with all of
the previous assets.

The `dfx install` process could be smarter, for example only uploading
changed assets. This would require more metadata, such as a hash per
content type/content blob.

These API methods are structured to facilitate efficient upload of many
assets within a single block:

-   `create_blobs` (call once)

-   `write_blob` (call many times concurrently)

-   `batch` (call once)

# Breaking Changes

This feature breaks the signature of the `store` method.

## Deprecation

This feature deprecates the `retrieve` method.

# Lifecycle

## Integration Plan

The JavaScript agent will need to change in order to use the new
interface.

The process that `dfx install` uses to synchronize assets to an asset
canister will be more complex.

## Maintenance Plan

The API operation parameters are passed as a record in order to
facilitate future changes.
