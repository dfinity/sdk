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
    public type ChunkId = Nat;
    public type EncodingId = Text;
    public type Key = Text;
    public type Path = Text;
    public type Commit = Bool;
    public type Contents = Blob;
    public type ContentEncoding = Text;
    public type ContentType = Text;
    public type Offset = Nat;
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
        #create_asset: CreateAssetArguments;
        #set_asset_content: SetAssetContentArguments;
        #unset_asset_content: UnsetAssetContentArguments;

        #delete_asset: DeleteAssetArguments;

        #clear: ClearArguments;
    };



    stable var authorized: [Principal] = [creator];

    let db: Tree.RBTree<Path, Contents> = Tree.RBTree(Text.compare);

  type AssetEncoding = {
    contentEncoding: Text;
    content: [Blob];
    totalLength: Nat;
  };

  type Asset = {
    contentType: Text;
    encodings: SHM.StableHashMap<Text, AssetEncoding>;
  };

  func getAssetEncoding(asset : Asset, acceptEncodings : [Text]) : ?AssetEncoding {
    for (acceptEncoding in acceptEncodings.vals()) {
      switch (encodings_manipulator.get(asset.encodings, acceptEncoding)) {
        case null {};
        case (?encodings) return ?encodings;
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

  type Chunk = {
    batch: Batch;
    content: Blob;
  };

  var nextBlobId = 1;
  let blobs = H.HashMap<Text, BlobBuffer>(7, Text.equal, Text.hash);

  var nextChunkId = 1;
  let chunks = H.HashMap<Int, Chunk>(7, Int.equal, Int.hash);

  func allocBlobId() : BlobId {
    let result = nextBlobId;
    nextBlobId += 1;
    Int.toText(result)
  };

  func createChunk(batch: Batch, content: Blob) : ChunkId {
    let chunkId = nextChunkId;
    nextChunkId += 1;
    let chunk : Chunk = {
      batch = batch;
      content = content;
    };
    chunks.put(chunkId, chunk);
    chunkId
  };

  var nextEncodingId = 1;
  let encodings = H.HashMap<Text, [var ?Blob]>(7, Text.equal, Text.hash);
  func allocEncodingId() : EncodingId {
    let result = nextEncodingId;
    nextEncodingId += 1;
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

  func takeChunk(chunkId: ChunkId): Result.Result<Blob, Text> {
    switch (chunks.remove(chunkId)) {
      case null #err("chunk not found");
      case (?chunk) #ok(chunk.content);
    }
  };

  type Batch = {
      expiry : Time;
  };

  // We track when each group of blobs should expire,
  // so that they don't consume space after an interrupted install.
  let BATCH_EXPIRY_NANOS = 5 * 60 * 1000 * 1000;
  var next_batch_id = 1;
  type Time = Int;
  let batches = H.HashMap<Int, Batch>(7, Int.equal, Int.hash);

  func startBatch(): BatchId {
    let batch_id = next_batch_id;
    next_batch_id += 1;
    let batch : Batch = {
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
        if (isSafe(caller)) {
            db.put(path, contents);
        } else {
            throw Error.reject("not authorized");
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
  }) : async ( {
    content: Blob;
    content_type: Text;
    content_encoding: Text;
    total_length: Nat;
  } ) {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("not found");
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
    key: Key;
    content_encoding: Text;
    index: Nat;
  }) : async ( {
    content: Blob
  }) {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("not found");
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

  func createBlob(batchId: BatchId, length: Nat32) : Result.Result<BlobId, Text> {
    let blobId = allocBlobId();

    let blob = Array.init<Nat8>(Nat32.toNat(length), 0);
    let blobBuffer = BlobBuffer(batchId, blob);

    blobs.put(blobId, blobBuffer);

    #ok(blobId)
  };

  func createEncoding(batch: Batch, chunks: Nat32) : Result.Result<EncodingId, Text> {
    let encodingId = allocEncodingId();
    let chunkArray = Array.init<?Blob>(Nat32.toNat(chunks), null);
    encodings.put(encodingId, chunkArray);

    #ok(encodingId)
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

  public func create_encodings( arg: {
    encoding_info: [ { chunks: Nat32 } ]
  } ) : async ( { encoding_ids: [EncodingId] } ) {
    let batch : Batch = {
      expiry = Time.now() + BATCH_EXPIRY_NANOS;
    };
    let createEncodingInBatch = func (arg: { chunks: Nat32 }) : Result.Result<EncodingId, Text> {
      createEncoding(batch, arg.chunks)
    };
    switch (Array.mapResult<{chunks: Nat32}, EncodingId, Text>(arg.encoding_info, createEncodingInBatch)) {
      case (#ok(ids)) { { encoding_ids = ids } };
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

  public func set_encoding_chunk( arg: {
    encoding_id: EncodingId;
    index: Nat;
    contents: Blob;
  } ) : async () {
    switch (encodings.get(arg.encoding_id)) {
      case null throw Error.reject("Encoding not found");
      case (?encodingChunks) encodingChunks[arg.index] := ?arg.contents;
    }
  };

  public func create_batch() : async ({
    batch_id: BatchId
  }) {
    {
      batch_id = startBatch();
    }
  };

  public func create_chunk( arg: {
    batch_id: BatchId;
    content: Blob;
  } ) : async ({
    chunk_id: ChunkId
  }) {
    switch (batches.get(arg.batch_id)) {
      case null throw Error.reject("batch not found");
      case (?batch) {
        {
          chunk_id = createChunk(batch, arg.content)
        }
      }
    }
  };

    public func batch(ops: [BatchOperationKind]) : async () {
        throw Error.reject("batch: not implemented");
    };

  public func create_asset(arg: CreateAssetArguments) : async () {
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

  func addBlobLength(acc: Nat, blob: Blob): Nat {
    acc + blob.size()
  };

  public func set_asset_content(arg: SetAssetContentArguments) : async () {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("Asset not found");
      case (?asset) {
        switch (Array.mapResult<ChunkId, Blob, Text>(arg.chunk_ids, takeChunk)) {
          case (#ok(chunks)) {
            let encoding : AssetEncoding = {
              contentEncoding = arg.content_encoding;
              content = chunks;
              totalLength = Array.foldLeft<Blob, Nat>(chunks, 0, addBlobLength);
            };

            encodings_manipulator.put(asset.encodings, arg.content_encoding, encoding);
          };
          case (#err(err)) throw Error.reject(err);
        };
      };
    };
  };

    public func unset_asset_content(op: UnsetAssetContentArguments) : async () {
        throw Error.reject("unset_asset_content: not implemented");
    };

    public func delete_asset(op: DeleteAssetArguments) : async () {
        throw Error.reject("delete_asset: not implemented");
    };

    public func clear(op: ClearArguments) : async () {
        throw Error.reject("clear: not implemented");
    };

    public func version_8() : async() {
    }
};
