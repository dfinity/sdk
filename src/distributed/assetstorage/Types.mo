import SHM "StableHashMap";
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
    encodings: SHM.StableHashMap<Text, AssetEncoding>;
  };

  public type Chunk = {
    batch: Batch;
    content: Blob;
  };

  public type Batch = {
      expiry : Time;
  };


};