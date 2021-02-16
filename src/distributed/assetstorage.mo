import Error "mo:base/Error";
import Iter "mo:base/Iter";
import Array "mo:base/Array";
import Text "mo:base/Text";
import Tree "mo:base/RBTree";

shared ({caller = creator}) actor class () {

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
       throw Error.reject("get: not implemented");
    };

    public func create_blobs( arg: {
            blob_info: [ {
                length: Nat32
            } ]
    } ) : async ( { blob_ids: [BlobId] } ) {
        throw Error.reject("create_blobs: not implemented");
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

    public func create(arg:{
            path: Path;
            contents: Contents;
            total_length: TotalLength;
            content_type: ContentType;
            content_encoding: ContentEncoding;
            commit: Commit}) : async () {
       throw Error.reject("not implemented");
    };

    public func write(path: Path, offset: Offset, contents: Contents) : async () {
       throw Error.reject("not implemented");
    };

    public func commit(path: Path) : async () {
       throw Error.reject("not implemented");
    };

    public func commit_many(paths: [Path]) : async () {
       throw Error.reject("not implemented");
    };

};
