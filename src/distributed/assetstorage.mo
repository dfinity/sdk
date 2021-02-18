import Array "mo:base/Array";
import Error "mo:base/Error";
import H "mo:base/HashMap";
import Int "mo:base/Int";
import Iter "mo:base/Iter";
import Nat8 "mo:base/Nat8";
import Nat32 "mo:base/Nat32";
import Result "mo:base/Result";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Tree "mo:base/RBTree";

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

    type Asset = {
      content_type: Text;
    };

    stable var asset_entries : [(Key, Asset)] = [];
    let assets = H.fromIter(asset_entries.vals(), 7, Text.equal, Text.hash);

    system func preupgrade() {
        asset_entries := Iter.toArray(assets.entries());
    };

    system func postupgrade() {
        asset_entries := [];
    };

    // blob data doesn't need to be stable
    class BlobBuffer(initBatchId: Nat, initBlob: [var Nat8]) {
        let batchId = initBatchId;
        let blob = initBlob;
    };

    var next_blob_id = 1;
    let blobs = H.HashMap<Text, BlobBuffer>(7, Text.equal, Text.hash);
    func alloc_blob_id() : BlobId {
        let result = next_blob_id;
        next_blob_id += 1;
        Int.toText(result)
    };

    // We track when each group of blobs should expire,
    // so that they don't consume space after an interrupted install.
    let BATCH_EXPIRY_NANOS = 5 * 1000 * 1000;
    var next_batch_id = 1;
    type Time = Int;
    let batch_expiry = H.HashMap<Int, Time>(7, Int.equal, Int.hash);

    func start_batch(): BatchId {
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

    public query func retrieve(path : Path) : async Contents {
        switch (db.get(path)) {
        case null throw Error.reject("not found");
        case (?contents) contents;
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

    public query func get(arg:{
            key: Key;
            accept_encodings: [Text]
    }) : async ( { contents: Blob; content_type: Text; content_encoding: Text } ) {
        switch (assets.get(arg.key)) {
        case null throw Error.reject("not found");
        case (?asset) throw Error.reject("found but not implemented");
        };
    };

    func createBlob(batchId: BatchId, length: Nat32) : Result.Result<BlobId, Text> {
        let blobId = alloc_blob_id();

        let blob = Array.init<Nat8>(Nat32.toNat(length), 0);
        let blobBuffer = BlobBuffer(batchId, blob);

        blobs.put(blobId, blobBuffer);

        #ok(blobId)
    };

    //type BlobParameters = {
    //    length: Nat32
    //};
    //type CreateBlobsResult = {
    //    blob_ids: [BlobId]
    //};
    public func create_blobs( arg: {
            blob_info: [ { length: Nat32 } ]
    } ) : async ( { blob_ids: [BlobId] } ) {
        let batch_id = start_batch();

        let createBlobInBatch = func (arg: { length: Nat32 }) : Result.Result<BlobId, Text> {
          createBlob(batch_id, arg.length)
        };

        switch(Array.mapResult<{length: Nat32}, BlobId, Text>(arg.blob_info, createBlobInBatch)) {
          case (#ok(ids)) { { blob_ids = ids } };
          case (#err(err)) throw Error.reject(err);
        }
    };

    public func write_blob( arg: {
            blob_id: BlobId;
            offset: Nat32;
            contents: Blob
    } ) : async () {
        throw Error.reject("write_blob: not implemented");
    };

    public func batch(ops: [BatchOperationKind]) : async() {
        throw Error.reject("batch: not implemented");
    };

    public func create_asset(op: CreateAssetOperation) : async () {
        throw Error.reject("create_asset: not implemented");
    };

    public func set_asset_content(op: SetAssetContentOperation) : async () {
        throw Error.reject("set_asset_content: not implemented");
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
};
