import Array "mo:base/Array";
import Buffer "mo:base/Buffer";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import H "mo:base/HashMap";
import Int "mo:base/Int";
import Iter "mo:base/Iter";
import Nat "mo:base/Nat";
import Nat8 "mo:base/Nat8";
import Nat32 "mo:base/Nat32";
import Result "mo:base/Result";
import T "assetstorage/Types";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Tree "mo:base/RBTree";
import U "assetstorage/Utils";
import Word8 "mo:base/Word8";


shared ({caller = creator}) actor class () {

  public type Path = Text;
  public type Contents = Blob;

  stable var authorized: [Principal] = [creator];

  stable var stableAssets : [(T.Key, T.StableAsset)] = [];
  let assets = H.fromIter(Iter.map(stableAssets.vals(), T.fromStableAssetEntry), 7, Text.equal, Text.hash);

  var nextChunkId = 1;
  let chunks = H.HashMap<Int, T.Chunk>(7, Int.equal, Int.hash);

  // We track when each group of blobs should expire,
  // so that they don't consume space after an interrupted install.
  let BATCH_EXPIRY_NANOS = 5 * 60 * 1000 * 1000 * 1000;
  var nextBatchId = 1;
  let batches = H.HashMap<Int, T.Batch>(7, Int.equal, Int.hash);

  system func preupgrade() {
    stableAssets := Iter.toArray(Iter.map(assets.entries(), T.fromAssetEntry));
  };

  system func postupgrade() {
    stableAssets := [];
  };

  func getAssetEncoding(asset : T.Asset, acceptEncodings : [Text]) : ?T.AssetEncoding {
    for (acceptEncoding in acceptEncodings.vals()) {
      switch (asset.encodings.get(acceptEncoding)) {
        case null {};
        case (?encodings) return ?encodings;
      }
    };
    null
  };

  func createChunk(batch: T.Batch, content: Blob) : T.ChunkId {
    let chunkId = nextChunkId;
    nextChunkId += 1;
    let chunk : T.Chunk = {
      batch = batch;
      content = content;
    };

    batch.expiry := Time.now() + BATCH_EXPIRY_NANOS;
    chunks.put(chunkId, chunk);

    chunkId
  };

  func takeChunk(chunkId: T.ChunkId): Result.Result<Blob, Text> {
    switch (chunks.remove(chunkId)) {
      case null #err("chunk not found");
      case (?chunk) #ok(chunk.content);
    }
  };


  func startBatch(): T.BatchId {
    let batch_id = nextBatchId;
    nextBatchId += 1;
    let batch : T.Batch = {
      var expiry = Time.now() + BATCH_EXPIRY_NANOS;
    };
    batches.put(batch_id, batch);
    batch_id
  };

  func expireBatches() : () {
    let now = Time.now();
    let batchesToDelete: H.HashMap<Int, T.Batch> = H.mapFilter<Int, T.Batch, T.Batch>(batches, Int.equal, Int.hash,
      func(k: Int, batch: T.Batch) : ?T.Batch {
        if (batch.expiry <= now)
          ?batch
        else
          null
      }
    );
    for ((k,_) in batchesToDelete.entries()) {
      Debug.print("delete expired batch " # Int.toText(k));

      batches.delete(k);
    };
    let chunksToDelete = H.mapFilter<Int, T.Chunk, T.Chunk>(chunks, Int.equal, Int.hash,
      func(k: Int, chunk: T.Chunk) : ?T.Chunk {
        if (chunk.batch.expiry <= now)
          ?chunk
        else
          null
      }
    );
    for ((k,_) in chunksToDelete.entries()) {
      Debug.print("delete expired chunk " # Int.toText(k));
      chunks.delete(k);
    };
  };

  public shared ({ caller }) func authorize(other: Principal) : async () {
    if (isSafe(caller)) {
      authorized := Array.append<Principal>(authorized, [other]);
    } else {
      throw Error.reject("not authorized");
    }
  };

  public shared ({ caller }) func store(path : Path, contents : Contents) : async () {
    if (isSafe(caller) == false) {
      throw Error.reject("not authorized");
    };

    let batch_id = startBatch();
    let chunk_id = switch (batches.get(batch_id)) {
      case null throw Error.reject("batch not found");
      case (?batch) createChunk(batch, contents)
    };

    let create_asset_args : T.CreateAssetArguments = {
      key = path;
      content_type = "application/octet-stream"
    };
    switch(createAsset(create_asset_args)) {
      case (#ok(())) {};
      case (#err(msg)) throw Error.reject(msg);
    };

    let set_asset_content_args : T.SetAssetContentArguments = {
      key = path;
      content_encoding = "identity";
      chunk_ids = [ chunk_id ];
    };
    switch(setAssetContent(set_asset_content_args)) {
      case (#ok(())) {};
      case (#err(msg)) throw Error.reject(msg);
    };
  };

  public query func retrieve(path : Path) : async Blob {
    switch (assets.get(path)) {
      case null throw Error.reject("not found");
      case (?asset) {
        switch (getAssetEncoding(asset, ["identity"])) {
          case null throw Error.reject("no such encoding");
          case (?encoding) {
            encoding.content[0]
          }
        };
      };
    }
  };

  public query func list() : async [Path] {
    let iter = Iter.map<(Text, T.Asset), Path>(assets.entries(), func (key, _) = key);
    Iter.toArray(iter)
  };

  func isSafe(caller: Principal) : Bool {
    return true;
    func eq(value: Principal): Bool = value == caller;
    Array.find(authorized, eq) != null
  };

  public query func get(arg:{
    key: T.Key;
    accept_encodings: [Text]
  }) : async ( {
    content: Blob;
    content_type: Text;
    content_encoding: Text;
    total_length: Nat;
  } ) {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("asset not found");
      case (?asset) {
        switch (getAssetEncoding(asset, arg.accept_encodings)) {
          case null throw Error.reject("no such encoding");
          case (?encoding) {
            {
              content = encoding.content[0];
              content_type = asset.contentType;
              content_encoding = encoding.contentEncoding;
              total_length = encoding.totalLength;
            }
          }
        };
      };
    };
  };

  public query func get_chunk(arg:{
    key: T.Key;
    content_encoding: Text;
    index: Nat;
  }) : async ( {
    content: Blob
  }) {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("asset not found");
      case (?asset) {
        switch (asset.encodings.get(arg.content_encoding)) {
          case null throw Error.reject("no such encoding");
          case (?encoding) {
            {
              content = encoding.content[arg.index];
            }
          }
        };
      };
    };
  };

  public shared ({ caller }) func create_batch(arg: {}) : async ({
    batch_id: T.BatchId
  }) {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    Debug.print("create_batch");
    expireBatches();
    {
      batch_id = startBatch();
    }
  };

  public shared ({ caller }) func create_chunk( arg: {
    batch_id: T.BatchId;
    content: Blob;
  } ) : async ({
    chunk_id: T.ChunkId
  }) {
    Debug.print("create_chunk(batch " # Int.toText(arg.batch_id) # ", " # Int.toText(arg.content.size()) # " bytes)");
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch (batches.get(arg.batch_id)) {
      case null throw Error.reject("batch not found");
      case (?batch) {
        {
          chunk_id = createChunk(batch, arg.content)
        }
      }
    }
  };

  public shared ({ caller }) func commit_batch(args: T.CommitBatchArguments) : async () {
    Debug.print("commit_batch (" # Int.toText(args.operations.size()) # ")");
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    for (op in args.operations.vals()) {

      let r : Result.Result<(), Text> = switch(op) {
        case (#CreateAsset(args)) { createAsset(args); };
        case (#SetAssetContent(args)) { setAssetContent(args); };
        case (#UnsetAssetContent(args)) { unsetAssetContent(args); };
        case (#DeleteAsset(args)) { deleteAsset(args); };
        case (#Clear(args)) { clearEverything(args); }
      };
      switch(r) {
        case (#ok(())) {};
        case (#err(msg)) throw Error.reject(msg);
      };
    }
  };

  public shared ({ caller }) func create_asset(arg: T.CreateAssetArguments) : async () {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch(createAsset(arg)) {
      case (#ok(())) {};
      case (#err(err)) throw Error.reject(err);
    };
  };

  func createAsset(arg: T.CreateAssetArguments) : Result.Result<(), Text> {
    Debug.print("createAsset(" # arg.key # ")");
    switch (assets.get(arg.key)) {
      case null {
        let asset : T.Asset = {
          contentType = arg.content_type;
          encodings = H.HashMap<Text, T.AssetEncoding>(7, Text.equal, Text.hash);
        };
        assets.put(arg.key, asset );
      };
      case (?asset) {
        if (asset.contentType != arg.content_type)
          return #err("create_asset: content type mismatch");
      }
    };
    #ok(())
  };

  public shared ({ caller }) func set_asset_content(arg: T.SetAssetContentArguments) : async () {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch(setAssetContent(arg)) {
      case (#ok(())) {};
      case (#err(err)) throw Error.reject(err);
    };
  };

  func addBlobLength(acc: Nat, blob: Blob): Nat {
    acc + blob.size()
  };

  func chunkLengthsMatch(chunks: [Blob]): Bool {
    if (chunks.size() == 0)
      return true;

    let expectedLength = chunks[0].size();
    for (i in Iter.range(1, chunks.size()-2)) {
      Debug.print("chunk at index " # Int.toText(i) # " has length " # Int.toText(chunks[i].size()) # " and expected is " # Int.toText(expectedLength) );
      if (chunks[i].size() != expectedLength) {
        Debug.print("chunk at index " # Int.toText(i) # " with length " # Int.toText(chunks[i].size()) # " does not match expected length " # Int.toText(expectedLength) );

        return false;
      }
    };
    //var i = 1;
    //var last = chunks.size() - 1;
    //while (i <= last) {
    //  if (chunks[i].size() != expectedLength)
    //    return false;
    //  i += 1;
    //};
    true
  };

  func setAssetContent(arg: T.SetAssetContentArguments) : Result.Result<(), Text> {
    Debug.print("setAssetContent(" # arg.key # ")");
    switch (assets.get(arg.key)) {
      case null #err("asset not found");
      case (?asset) {
        switch (Array.mapResult<T.ChunkId, Blob, Text>(arg.chunk_ids, takeChunk)) {
          case (#ok(chunks)) {
            if (chunkLengthsMatch(chunks) == false) {
              #err("chunk lengths do not match the size of the first chunk")
            } else {
              let encoding : T.AssetEncoding = {
                contentEncoding = arg.content_encoding;
                content = chunks;
                totalLength = Array.foldLeft<Blob, Nat>(chunks, 0, addBlobLength);
              };

              asset.encodings.put(arg.content_encoding, encoding);
              #ok(());
            };
          };
          case (#err(err)) #err(err);
        };
      };
    }
  };

  public shared ({ caller }) func unset_asset_content(args: T.UnsetAssetContentArguments) : async () {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch(unsetAssetContent(args)) {
      case (#ok(())) {};
      case (#err(err)) throw Error.reject(err);
    };
  };

  func unsetAssetContent(args: T.UnsetAssetContentArguments) : Result.Result<(), Text> {
    #err("unset_asset_content: not implemented");
  };

  public shared ({ caller }) func delete_asset(args: T.DeleteAssetArguments) : async () {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch(deleteAsset(args)) {
      case (#ok(())) {};
      case (#err(err)) throw Error.reject(err);
    };
  };

  func deleteAsset(args: T.DeleteAssetArguments) : Result.Result<(), Text> {
    Debug.print("deleteAsset(" # args.key # ")");
    if (assets.size() > 0) {   // avoid div/0 bug   https://github.com/dfinity/motoko-base/issues/228
      assets.delete(args.key);
    };
    #ok(())
  };

  public shared ({ caller }) func clear(args: T.ClearArguments) : async () {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch(clearEverything(args)) {
      case (#ok(())) {};
      case (#err(err)) throw Error.reject(err);
    };
  };

  func clearEverything(args: T.ClearArguments) : Result.Result<(), Text> {
    stableAssets := [];
    U.clearHashMap(assets);

    nextBatchId := 1;
    U.clearHashMap(batches);

    nextChunkId := 1;
    U.clearHashMap(chunks);

    #ok(())
  };

  public func version_14() : async() {
  }
};
