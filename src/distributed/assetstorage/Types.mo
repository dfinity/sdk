import H "mo:base/HashMap";
import Iter "mo:base/Iter";
import Text "mo:base/Text";
import Time "mo:base/Time";

module Types {
  public type BatchId = Nat;
  public type BlobId = Text;
  public type ChunkId = Nat;
  public type ContentEncoding = Text;
  public type ContentType = Text;
  public type EncodingId = Text;
  public type Key = Text;
  public type Offset = Nat;
  public type Time = Int;
  public type TotalLength = Nat;

  public type CreateAssetArguments = {
    key: Key;
    content_type: Text;
  };

  public type SetAssetContentArguments = {
    key: Key;
    content_encoding: Text;
    chunk_ids: [ChunkId]
  };

  public type UnsetAssetContentArguments = {
    key: Key;
    content_encoding: Text;
  };

  public type DeleteAssetArguments = {
    key: Key;
  };

  public type ClearArguments = {
  };

  public type BatchOperationKind = {
    #CreateAsset: CreateAssetArguments;
    #SetAssetContent: SetAssetContentArguments;
    #UnsetAssetContent: UnsetAssetContentArguments;

    #DeleteAsset: DeleteAssetArguments;

    #Clear: ClearArguments;
  };

  public type CommitBatchArguments = {
    batch_id: BatchId;
    operations: [BatchOperationKind];
  };

  public type AssetEncoding = {
    contentEncoding: Text;
    content: [Blob];
    totalLength: Nat;
  };

  public type Asset = {
    contentType: Text;
    encodings: H.HashMap<Text, AssetEncoding>;
  };

  public type StableAsset = {
    contentType: Text;
    encodings: [(Text, AssetEncoding)];
  };

  public func fromAssetEntry((k: Key, v: Asset)) : ((Key, StableAsset)) {
    let fa : StableAsset = {
      contentType = v.contentType;
      encodings = Iter.toArray(v.encodings.entries());
    };
    (k, fa)
  };

  public func fromStableAssetEntry((k: Key, v: StableAsset)) : ((Key, Asset)) {
    let a : Asset = {
      contentType = v.contentType;
      encodings = H.fromIter(v.encodings.vals(), 7, Text.equal, Text.hash);
    };
    (k, a)
  };

  public type Chunk = {
    batch: Batch;
    content: Blob;
  };

  object batch {
    let expiryNanos = 300_000_000_000; // 5 * 60 * 1000 * 1000 * 1000;

    public func nextExpireTime() : Time {
      Time.now() + expiryNanos
    }
  };

  public class Batch() {
      var expiresAt : Time = batch.nextExpireTime();

      public func refreshExpiry() {
        expiresAt := batch.nextExpireTime();
      };

      public func expired(asOf : Time) : Bool {
        expiresAt <= asOf
      };
  };
};