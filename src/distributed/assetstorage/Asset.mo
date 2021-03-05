import HashMap "mo:base/HashMap";
import Iter "mo:base/Iter";
import Text "mo:base/Text";

import T "Types";
import U "Utils";

module {
  public type AssetEncoding = {
    contentEncoding: Text;
    content: [Blob];
    totalLength: Nat;
  };
  public class Asset(
    initContentType: Text,
    initEncodings: HashMap.HashMap<Text, AssetEncoding>
  ) {
    public let contentType = initContentType;
    public let encodings = initEncodings;

    public func getEncoding(acceptEncodings : [Text]) : ?AssetEncoding {
      for (acceptEncoding in acceptEncodings.vals()) {
        switch (encodings.get(acceptEncoding)) {
          case null {};
          case (?encoding) return ?encoding;
        }
      };
      null
    };
  };
  /*
  //public type Asset = {
  //  contentType: Text;
   // encodings: HashMap.HashMap<Text, AssetEncoding>;
  //};

  public func getAssetEncoding(asset : Asset, acceptEncodings : [Text]) : ?AssetEncoding {
    for (acceptEncoding in acceptEncodings.vals()) {
      switch (asset.encodings.get(acceptEncoding)) {
        case null {};
        case (?encoding) return ?encoding;
      }
    };
    null
  };*/

  public type StableAsset = {
    contentType: Text;
    encodings: [(Text, AssetEncoding)];
  };

  public func toStableAssetEntry((k: T.Key, v: Asset)) : ((T.Key, StableAsset)) {
    let sa : StableAsset = {
      contentType = v.contentType;
      encodings = Iter.toArray(v.encodings.entries());
    };
    (k, sa)
  };

  public func toAssetEntry((k: T.Key, v: StableAsset)) : ((T.Key, Asset)) {
    let a = Asset(
      v.contentType,
      HashMap.fromIter(v.encodings.vals(), 7, Text.equal, Text.hash)
    );
    (k, a)
  };

}