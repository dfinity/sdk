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
import SHM "assetstorage/StableHashMap";
import T "assetstorage/Types";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Tree "mo:base/RBTree";
import Word8 "mo:base/Word8";


shared ({caller = creator}) actor class () {

  // old interface:
  public type Path = Text;
  public type Contents = Blob;

  // new hotness


  stable var authorized: [Principal] = [creator];


  func getAssetEncoding(asset : T.Asset, acceptEncodings : [Text]) : ?T.AssetEncoding {
    for (acceptEncoding in acceptEncodings.vals()) {
      switch (encodings_manipulator.get(asset.encodings, acceptEncoding)) {
        case null {};
        case (?encodings) return ?encodings;
      }
    };
    null
  };


  //stable var asset_entries : [(T.Key, T.Asset)] = [];
  //let assets = H.fromIter(asset_entries.vals(), 7, Text.equal, Text.hash);
  stable let assets : SHM.StableHashMap<T.Key, T.Asset> = SHM.StableHashMap<T.Key, T.Asset>();
  let assets_manipulator = SHM.StableHashMapManipulator<T.Key, T.Asset>(7, Text.equal, Text.hash);
  let encodings_manipulator = SHM.StableHashMapManipulator<Text, T.AssetEncoding>(7, Text.equal, Text.hash);

  system func preupgrade() {
    //asset_entries := Iter.toArray(assets.entries());
  };

  system func postupgrade() {
    //asset_entries := [];
  };

  var nextChunkId = 1;
  let chunks = H.HashMap<Int, T.Chunk>(7, Int.equal, Int.hash);

  func createChunk(batch: T.Batch, content: Blob) : T.ChunkId {
    let chunkId = nextChunkId;
    nextChunkId += 1;
    let chunk : T.Chunk = {
      batch = batch;
      content = content;
    };
    chunks.put(chunkId, chunk);
    chunkId
  };

  //var nextEncodingId = 1;
  //let encodings = H.HashMap<Text, [var ?Blob]>(7, Text.equal, Text.hash);

  func takeChunk(chunkId: T.ChunkId): Result.Result<Blob, Text> {
    switch (chunks.remove(chunkId)) {
      case null #err("chunk not found");
      case (?chunk) #ok(chunk.content);
    }
  };

  // We track when each group of blobs should expire,
  // so that they don't consume space after an interrupted install.
  let BATCH_EXPIRY_NANOS = 5 * 60 * 1000 * 1000;
  var next_batch_id = 1;
  let batches = H.HashMap<Int, T.Batch>(7, Int.equal, Int.hash);

  func startBatch(): T.BatchId {
    let batch_id = next_batch_id;
    next_batch_id += 1;
    let batch : T.Batch = {
      expiry = Time.now() + BATCH_EXPIRY_NANOS;
    };
    batches.put(batch_id, batch);
    batch_id
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
    switch (assets_manipulator.get(assets, path)) {
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
    let iter = Iter.map<(Text, T.Asset), Path>(assets_manipulator.entries(assets), func (key, _) = key);
    Iter.toArray(iter)
  };

  func isSafe(caller: Principal) : Bool {
    //return true;
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
    switch (assets_manipulator.get(assets, arg.key)) {
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
    switch (assets_manipulator.get(assets, arg.key)) {
      case null throw Error.reject("asset not found");
      case (?asset) {
        switch (encodings_manipulator.get(asset.encodings, arg.content_encoding)) {
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
    switch (assets_manipulator.get(assets, arg.key)) {
      case null {
        let asset : T.Asset = {
          contentType = arg.content_type;
          encodings = SHM.StableHashMap<Text, T.AssetEncoding>();
        };
        assets_manipulator.put(assets, arg.key, asset );
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

  func setAssetContent(arg: T.SetAssetContentArguments) : Result.Result<(), Text> {
    Debug.print("setAssetContent(" # arg.key # ")");
    switch (assets_manipulator.get(assets, arg.key)) {
      case null #err("asset not found");
      case (?asset) {
        switch (Array.mapResult<T.ChunkId, Blob, Text>(arg.chunk_ids, takeChunk)) {
          case (#ok(chunks)) {
            let encoding : T.AssetEncoding = {
              contentEncoding = arg.content_encoding;
              content = chunks;
              totalLength = Array.foldLeft<Blob, Nat>(chunks, 0, addBlobLength);
            };

            encodings_manipulator.put(asset.encodings, arg.content_encoding, encoding);
            #ok(());
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
    assets_manipulator.delete(assets, args.key);
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
    #err("clear: not implemented")
  };

  public func version_13() : async() {
  }
};
