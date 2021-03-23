module Types {
  public type Contents = Blob;
  public type Path = Text;

  public type BatchId = Nat;
  public type ChunkId = Nat;
  public type Key = Text;
  public type Time = Int;

  public type AssetEncodingDetails = {
    content_encoding: Text;
    sha256: ?Blob;
  };

  public type AssetDetails = {
    key: Key;
    content_type: Text;
    encodings: [AssetEncodingDetails];
  };

  public type CreateAssetArguments = {
    key: Key;
    content_type: Text;
  };

  public type SetAssetContentArguments = {
    key: Key;
    content_encoding: Text;
    chunk_ids: [ChunkId];
    sha256: ?Blob;
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

  public type HeaderField = (Text, Text);

  public type HttpRequest = {
    method: Text;
    url: Text;
    headers: [HeaderField];
    body: Blob;
  };
  public type HttpResponse = {
    status_code: Nat16;
    headers: [HeaderField];
    body: Blob;
    next_token: ?HttpNextToken;
  };

  public type HttpNextToken = {
      key: Text;
      content_encoding: Text;
      index: Nat;
      sha256: ?Blob;
  };

  public type HttpNextResponse = {
    body: Blob;
    next_token: ?HttpNextToken;
  };
};
