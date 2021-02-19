import Array "mo:base/Array";
import Buffer "mo:base/Buffer";
import Error "mo:base/Error";
import H "mo:base/HashMap";
import Int "mo:base/Int";
import Iter "mo:base/Iter";
import Nat "mo:base/Nat";
import Nat8 "mo:base/Nat8";
import Nat32 "mo:base/Nat32";
import Result "mo:base/Result";
import SHM "StableHashMap";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Tree "mo:base/RBTree";
import Word8 "mo:base/Word8";

shared ({caller = creator}) actor class () {

    public type BatchId = Nat;
    public type BlobId = Text;
    public type Key = Text;
    public type Path = Text;
    public type Commit = Bool;
    public type Contents = Blob;
    public type ContentEncoding = Text;
    public type ContentType = Text;
    public type Offset = Nat;
    public type TotalLength = Nat;


    public type CreateAssetOperation = {
        key: Key;
        content_type: Text;
    };
    public type SetAssetContentOperation = {
        key: Key;
        content_encoding: Text;
        blob_id: BlobId;
    };
    public type UnsetAssetContentOperation = {
        key: Key;
        content_encoding: Text;
    };
    public type DeleteAssetOperation = {
        key: Key;
    };
    public type ClearOperation = {
    };

    public type BatchOperationKind = {
        #create: CreateAssetOperation;
        #set_content: SetAssetContentOperation;
        #unset_content: UnsetAssetContentOperation;

        #delete: DeleteAssetOperation;
        #clear: ClearOperation;
    };



    stable var authorized: [Principal] = [creator];

    let db: Tree.RBTree<Path, Contents> = Tree.RBTree(Text.compare);

  type AssetEncoding = {
    contentEncoding: Text;
    content: [Nat8];
  };

  type Asset = {
    contentType: Text;
    encodings: SHM.StableHashMap<Text, AssetEncoding>;
  };

  func getAssetEncoding(asset : Asset, acceptEncodings : [Text]) : ?AssetEncoding {
    for (acceptEncoding in acceptEncodings.vals()) {
      switch (encodings_manipulator.get(asset.encodings, acceptEncoding)) {
        case null {};
        case (?assetEncoding) return ?assetEncoding;
      }
    };
    null
  };

    stable var asset_entries : [(Key, Asset)] = [];
    let assets = H.fromIter(asset_entries.vals(), 7, Text.equal, Text.hash);
    let assets_manipulator = SHM.StableHashMapManipulator<Key, Asset>(7, Text.equal, Text.hash);
    let encodings_manipulator = SHM.StableHashMapManipulator<Text, AssetEncoding>(7, Text.equal, Text.hash);

    system func preupgrade() {
        asset_entries := Iter.toArray(assets.entries());
    };

    system func postupgrade() {
        asset_entries := [];
    };

  // blob data doesn't need to be stable
  class BlobBuffer(initBatchId: Nat, initBuffer: [var Nat8]) {
    let batchId = initBatchId;
    let buffer = initBuffer;

    public func setData(offset: Nat32, data: Blob): Result.Result<(), Text> {
      var index: Nat = Nat32.toNat(offset);

      if (index + data.size() > buffer.size()) {
        #err("overflow: offset " # Nat32.toText(offset) #
          " + data size " # Nat.toText(data.size()) #
          " exceeds blob size of " # Nat.toText(buffer.size()))
      } else {
        for (b in data.bytes()) {
          buffer[index] := Nat8.fromNat(Word8.toNat(b));
          index += 1;
        };
        #ok()
      }
    };

    public func takeBuffer() : [Nat8] {
      let x = Array.freeze(buffer);
      x
    };
  };

  var nextBlobId = 1;
  let blobs = H.HashMap<Text, BlobBuffer>(7, Text.equal, Text.hash);

  func allocBlobId() : BlobId {
    let result = nextBlobId;
    nextBlobId += 1;
    Int.toText(result)
  };

  func takeBlob(blobId: BlobId) : ?[Nat8] {
    switch (blobs.remove(blobId)) {
      case null null;
      case (?blobBuffer) {
        let b: [Nat8] = blobBuffer.takeBuffer();
        //let blob : Blob = b;
        ?b
      }
    }
  };

  // We track when each group of blobs should expire,
  // so that they don't consume space after an interrupted install.
  let BATCH_EXPIRY_NANOS = 5 * 1000 * 1000;
  var next_batch_id = 1;
  type Time = Int;
  let batch_expiry = H.HashMap<Int, Time>(7, Int.equal, Int.hash);

  func startBatch(): BatchId {
    let batch_id = next_batch_id;
    next_batch_id += 1;
    let expires = Time.now() + BATCH_EXPIRY_NANOS;
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
        if (isSafe(caller)) {
            db.put(path, contents);
        } else {
            throw Error.reject("not authorized");
        };
    };

  //public query func retrieveX(path : Path) : async Contents {
  //  let arg = {
  //    key = path;
  //    accept_encodings = [];
  //  };
  //  let x = get(arg);
  //  x.contents
  //};
  public query func retrieve(path : Path) : async [Nat8] {
    switch (assets.get(path)) {
    case null throw Error.reject("not found");
    case (?asset) {
      switch (getAssetEncoding(asset, [])) {
        case null throw Error.reject("no such encoding");
        case (?encoding) {
          encoding.content
        }
      };
      };
    };
  };

    public query func list() : async [Path] {
        let iter = Iter.map<(Path, Contents), Path>(db.entries(), func (path, _) = path);
        Iter.toArray(iter)
    };

    func isSafe(caller: Principal) : Bool {
        func eq(value: Principal): Bool = value == caller;
        Array.find(authorized, eq) != null
    };
  public query func get2() : async( { contents: Blob } ) {
    throw Error.reject("nyi");
  };

  public query func get(arg:{
    key: Key;
    accept_encodings: [Text]
  }) : async ( { contents: [Nat8]; content_type: Text; content_encoding: Text } ) {
    switch (assets.get(arg.key)) {
    case null throw Error.reject("not found");
    case (?asset) {
      switch (getAssetEncoding(asset, arg.accept_encodings)) {
        case null throw Error.reject("no such encoding");
        case (?encoding) {
          {
            contents = encoding.content;
            content_type = asset.contentType;
            content_encoding = encoding.contentEncoding;
          }
        }
      };
      };
    };
  };

  //func arrayToBlob(a : [Word8]) : Blob {
  //  a
  //};


  //func nat8ArrayToBlob(a : [Nat8]) : Blob {
  //  a
  //};

  func createBlob(batchId: BatchId, length: Nat32) : Result.Result<BlobId, Text> {
    let blobId = allocBlobId();

    let blob = Array.init<Nat8>(Nat32.toNat(length), 0);
    let blobBuffer = BlobBuffer(batchId, blob);

    blobs.put(blobId, blobBuffer);

    #ok(blobId)
  };

  public func create_blobs( arg: {
    blob_info: [ { length: Nat32 } ]
  } ) : async ( { blob_ids: [BlobId] } ) {
    let batchId = startBatch();

    let createBlobInBatch = func (arg: { length: Nat32 }) : Result.Result<BlobId, Text> {
      createBlob(batchId, arg.length)
    };

    switch (Array.mapResult<{length: Nat32}, BlobId, Text>(arg.blob_info, createBlobInBatch)) {
      case (#ok(ids)) { { blob_ids = ids } };
      case (#err(err)) throw Error.reject(err);
    }
  };

  public func write_blob( arg: {
    blob_id: BlobId;
    offset: Nat32;
    contents: Blob
  } ) : async () {
    switch (blobs.get(arg.blob_id)) {
      case null throw Error.reject("Blob not found");
      case (?blobBuffer) {
        switch (blobBuffer.setData(arg.offset, arg.contents)) {
          case (#ok) {};
          case (#err(text)) throw Error.reject(text);
        }
      };
    }
  };

    public func batch(ops: [BatchOperationKind]) : async () {
        throw Error.reject("batch: not implemented");
    };

  public func create_asset(arg: CreateAssetOperation) : async () {
    switch (assets.get(arg.key)) {
      case null {
        let asset : Asset = {
          contentType = arg.content_type;
          encodings = SHM.StableHashMap<Text, AssetEncoding>();
        };
        assets.put( (arg.key, asset) );
      };
      case (?asset) {
        if (asset.contentType != arg.content_type)
          throw Error.reject("create_asset: content type mismatch");

      }
    }
  };

  public func set_asset_content(arg: SetAssetContentOperation) : async () {
    switch (assets.get(arg.key), takeBlob(arg.blob_id)) {
      case (null,null) throw Error.reject("Asset and Blob not found");
      case (null,?blob) throw Error.reject("Asset not found");
      case (?asset,null) throw Error.reject("Blob not found");
      case (?asset,?blob) {
        let encoding : AssetEncoding = {
          contentEncoding = arg.content_encoding;
          content = blob;
        };

        encodings_manipulator.put(asset.encodings, arg.content_encoding, encoding);
      };
    };
  };

    public func unset_asset_content(op: UnsetAssetContentOperation) : async () {
        throw Error.reject("unset_asset_content: not implemented");
    };

    public func delete_asset(op: DeleteAssetOperation) : async () {
        throw Error.reject("delete_asset: not implemented");
    };

    public func clear(op: ClearOperation) : async () {
        throw Error.reject("clear: not implemented");
    };

    public func version_4() : async() {
    }
};
