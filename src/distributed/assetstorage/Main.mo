import Array "mo:base/Array";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import HashMap "mo:base/HashMap";
import Int "mo:base/Int";
import Iter "mo:base/Iter";
import Nat "mo:base/Nat";
import Nat8 "mo:base/Nat8";
import Result "mo:base/Result";
import Text "mo:base/Text";
import Time "mo:base/Time";

import A "Asset";
import B "Batch";
import C "Chunk";
import T "Types";
import U "Utils";


shared ({caller = creator}) actor class () {

  stable var authorized: [Principal] = [creator];

  stable var stableAssets : [(T.Key, A.StableAsset)] = [];
  let assets = HashMap.fromIter(Iter.map(stableAssets.vals(), A.toAssetEntry), 7, Text.equal, Text.hash);

  let chunks = C.Chunks();
  let batches = B.Batches();

  system func preupgrade() {
    stableAssets := Iter.toArray(Iter.map(assets.entries(), A.toStableAssetEntry));
  };

  system func postupgrade() {
    stableAssets := [];
  };

  public shared ({ caller }) func authorize(other: Principal) : async () {
    if (isSafe(caller)) {
      authorized := Array.append<Principal>(authorized, [other]);
    } else {
      throw Error.reject("not authorized");
    }
  };

  // Retrieve an asset's contents by name.
  // Rejects requests for assets composed of more than one chunk.
  // To handle larger assets, use get() and get_chunk().
  public query func retrieve(path : T.Path) : async T.Contents {
    switch (assets.get(path)) {
      case null throw Error.reject("not found");
      case (?asset) {
        switch (asset.getEncoding("identity")) {
          case null throw Error.reject("no identity encoding");
          case (?encoding) {
            if (encoding.content.size() > 1)
              throw Error.reject("Asset too large.  Use get() and get_chunk() instead.");
            encoding.content[0];
          }
        };
      };
    }
  };

  // Store a content encoding for an asset.  Does not remove other content encodings.
  // If the contents exceed the message ingress limit,
  // use create_batch(), create_chunk(), commit_batch() instead.
  public shared ({ caller }) func store(arg:{
    key: T.Key;
    content_type: Text;
    content_encoding: Text;
    content: Blob;
    sha256: ?Blob;
  }) : async () {
    if (isSafe(caller) == false) {
      throw Error.reject("not authorized");
    };

    let batch = batches.create();
    let chunkId = chunks.create(batch, arg.content);

    let create_asset_args : T.CreateAssetArguments = {
      key = arg.key;
      content_type = arg.content_type;
    };
    switch(createAsset(create_asset_args)) {
      case (#ok(())) {};
      case (#err(msg)) throw Error.reject(msg);
    };

    let args : T.SetAssetContentArguments = {
      key = arg.key;
      content_encoding = arg.content_encoding;
      chunk_ids = [ chunkId ];
      sha256 = arg.sha256;
    };
    switch(setAssetContent(args)) {
      case (#ok(())) {};
      case (#err(msg)) throw Error.reject(msg);
    };
  };

  func entryToAssetDetails((key: T.Key, asset: A.Asset)) : T.AssetDetails {
    let assetEncodings = Iter.toArray(
      Iter.map<(Text, A.AssetEncoding), T.AssetEncodingDetails>(
        asset.encodingEntries(), entryToAssetEncodingDetails
      )
    );

    {
      key = key;
      content_type = asset.contentType;
      encodings = assetEncodings;
    }
  };

  func entryToAssetEncodingDetails((name: Text, assetEncoding: A.AssetEncoding)) : T.AssetEncodingDetails {
    {
      content_encoding = assetEncoding.contentEncoding;
      sha256 = assetEncoding.sha256;
    }
  };

  public query func list(arg:{}) : async [T.AssetDetails] {
    let iter = Iter.map<(Text, A.Asset), T.AssetDetails>(assets.entries(), entryToAssetDetails);
    Iter.toArray(iter)
  };

  func isSafe(caller: Principal) : Bool {
    func eq(value: Principal): Bool = value == caller;
    Array.find(authorized, eq) != null
  };

  // 1. Choose a content encoding from among the accepted encodings.
  // 2. Return its content, or the first chunk of its content.
  //
  // If content.size() > total_length, caller must call get_chunk() get the rest of the content.
  // All chunks except the last will have the same size as the first chunk.
  public query func get(arg:{
    key: T.Key;
    accept_encodings: [Text]
  }) : async ( {
    content: Blob;
    content_type: Text;
    content_encoding: Text;
    total_length: Nat;
    sha256: ?Blob;
  } ) {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("asset not found");
      case (?asset) {
        switch (asset.chooseEncoding(arg.accept_encodings)) {
          case null throw Error.reject("no such encoding");
          case (?encoding) {
            {
              content = encoding.content[0];
              content_type = asset.contentType;
              content_encoding = encoding.contentEncoding;
              total_length = encoding.totalLength;
              sha256 = encoding.sha256;
            }
          }
        };
      };
    };
  };

  // Get subsequent chunks of an asset encoding's content, after get().
  public query func get_chunk(arg:{
    key: T.Key;
    content_encoding: Text;
    index: Nat;
    sha256: ?Blob;
  }) : async ( {
    content: Blob
  }) {
    switch (assets.get(arg.key)) {
      case null throw Error.reject("asset not found");
      case (?asset) {
        switch (asset.getEncoding(arg.content_encoding)) {
          case null throw Error.reject("no such encoding");
          case (?encoding) {
            switch (arg.sha256, encoding.sha256) {
              case (?expected, ?actual) {
                if (expected != actual)
                  throw Error.reject("sha256 mismatch");
              };
              case (?expected, null) throw Error.reject("sha256 specified but asset encoding has none");
              case (null, _) {};
            };

            {
              content = encoding.content[arg.index];
            }
          }
        };
      };
    };
  };

  // All chunks are associated with a batch until committed with commit_batch.
  public shared ({ caller }) func create_batch(arg: {}) : async ({
    batch_id: T.BatchId
  }) {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    batches.deleteExpired();
    chunks.deleteExpired();

    {
      batch_id = batches.create().batchId;
    }
  };

  public shared ({ caller }) func create_chunk( arg: {
    batch_id: T.BatchId;
    content: Blob;
  } ) : async ({
    chunk_id: T.ChunkId
  }) {
    //Debug.print("create_chunk(batch " # Int.toText(arg.batch_id) # ", " # Int.toText(arg.content.size()) # " bytes)");
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    let chunkId = switch (batches.get(arg.batch_id)) {
      case null throw Error.reject("batch not found");
      case (?batch) chunks.create(batch, arg.content)
    };

    {
      chunk_id = chunkId;
    }
  };

  public shared ({ caller }) func commit_batch(args: T.CommitBatchArguments) : async () {
    //Debug.print("commit_batch (" # Int.toText(args.operations.size()) # ")");
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    for (op in args.operations.vals()) {
      let r : Result.Result<(), Text> = switch(op) {
        case (#CreateAsset(args)) { createAsset(args); };
        case (#SetAssetContent(args)) { setAssetContent(args); };
        case (#UnsetAssetContent(args)) { unsetAssetContent(args); };
        case (#DeleteAsset(args)) { deleteAsset(args); };
        case (#Clear(args)) { doClear(args); }
      };
      switch(r) {
        case (#ok(())) {};
        case (#err(msg)) throw Error.reject(msg);
      };
    };
    batches.delete(args.batch_id);
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
    //Debug.print("createAsset(" # arg.key # ")");
    switch (assets.get(arg.key)) {
      case null {
        let asset = A.Asset(
          arg.content_type,
          HashMap.HashMap<Text, A.AssetEncoding>(7, Text.equal, Text.hash)
        );
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

  func chunkLengthsMatch(chunks: [Blob]): Bool {
    if (chunks.size() > 2) {
      let expectedLength = chunks[0].size();
      for (i in Iter.range(1, chunks.size()-2)) {
        //Debug.print("chunk at index " # Int.toText(i) # " has length " # Int.toText(chunks[i].size()) # " and expected is " # Int.toText(expectedLength) );
        if (chunks[i].size() != expectedLength) {
          //Debug.print("chunk at index " # Int.toText(i) # " with length " # Int.toText(chunks[i].size()) # " does not match expected length " # Int.toText(expectedLength) );

          return false;
        }
      };
    };
    true
  };

  func setAssetContent(arg: T.SetAssetContentArguments) : Result.Result<(), Text> {
    //Debug.print("setAssetContent(" # arg.key # ")");
    switch (assets.get(arg.key)) {
      case null #err("asset not found");
      case (?asset) {
        switch (Array.mapResult<T.ChunkId, Blob, Text>(arg.chunk_ids, chunks.take)) {
          case (#ok(chunks)) {
            if (chunkLengthsMatch(chunks) == false) {
              #err(arg.key # "(" # arg.content_encoding # "): chunk lengths do not match the size of the first chunk")
            } else if (chunks.size() == 0) {
              #err(arg.key # "(" # arg.content_encoding # "): must have at least one chunk")
            } else {
              let encoding : A.AssetEncoding = {
                contentEncoding = arg.content_encoding;
                content = chunks;
                totalLength = Array.foldLeft<Blob, Nat>(chunks, 0, func (acc: Nat, blob: Blob): Nat {
                  acc + blob.size()
                });
                sha256 = arg.sha256;
              };
              #ok(asset.setEncoding(arg.content_encoding, encoding));
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
    //Debug.print("unsetAssetContent(" # args.key # ")");
    switch (assets.get(args.key)) {
      case null #err("asset not found");
      case (?asset) {
        asset.unsetEncoding(args.content_encoding);
        #ok(())
      };
    };
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
    //Debug.print("deleteAsset(" # args.key # ")");
    if (assets.size() > 0) {   // avoid div/0 bug   https://github.com/dfinity/motoko-base/issues/228
      assets.delete(args.key);
    };
    #ok(())
  };

  public shared ({ caller }) func clear(args: T.ClearArguments) : async () {
    if (isSafe(caller) == false)
      throw Error.reject("not authorized");

    switch(doClear(args)) {
      case (#ok(())) {};
      case (#err(err)) throw Error.reject(err);
    };
  };

  func doClear(args: T.ClearArguments) : Result.Result<(), Text> {
    stableAssets := [];
    U.clearHashMap(assets);

    batches.reset();
    chunks.reset();

    #ok(())
  };

  public query func http_request(request: T.HttpRequest): async T.HttpResponse {
    let key = getKey(request.url);

    let assetAndEncoding: ?(A.Asset, A.AssetEncoding) = switch (getAssetAndEncoding(key)) {
      case (?found) ?found;
      case (null) getAssetAndEncoding("/index.html");
    };


    switch (assetAndEncoding) {
      case null {{ status_code = 404; headers = []; body = ""; next_token = null }};
      case (?(asset, assetEncoding)) {
        {
          status_code = 200;
          headers = [];
          body = assetEncoding.content[0];
          next_token = makeNextToken(key, assetEncoding, 0);
        }
      };
    }
  };

  // Get subsequent chunks of an asset encoding's content, after http_request().
  // Like get_chunk, but converts url to key
  public query func http_request_next(token: T.HttpNextToken) : async T.HttpNextResponse {
    switch (assets.get(token.key)) {
      case null throw Error.reject("asset not found");
      case (?asset) {
        switch (asset.getEncoding(token.content_encoding)) {
          case null throw Error.reject("no such encoding");
          case (?encoding) {
            switch (token.sha256, encoding.sha256) {
              case (?expected, ?actual) {
                if (expected != actual)
                  throw Error.reject("sha256 mismatch");
              };
              case (?expected, null) throw Error.reject("sha256 specified but asset encoding has none");
              case (null, _) {};
            };

            {
              body = encoding.content[token.index];
              next_token = makeNextToken(token.key, encoding, token.index);
            }
          }
        };
      };
    };
  };

  private func makeNextToken(key: T.Key, assetEncoding: A.AssetEncoding, lastIndex: Nat): ?T.HttpNextToken {
    if (lastIndex + 1 < assetEncoding.content.size()) {
      ?{
        key = key;
        content_encoding = assetEncoding.contentEncoding;
        index = lastIndex + 1;
        sha256 = assetEncoding.sha256;
      };
    } else {
      null;
    };
  };

  private func getKey(uri: Text): Text {
    let splitted = Text.split(uri, #char '?');
    let array = Iter.toArray<Text>(splitted);
    let path = array[0];
    path
  };

  private func getAssetAndEncoding(path: Text): ?(A.Asset, A.AssetEncoding) {
    switch (assets.get(path)) {
      case null null;
      case (?asset) {
        switch (asset.getEncoding("identity")) {
          case null null;
          case (?assetEncoding) ?(asset, assetEncoding);
        }
      }
    }
  };

};
