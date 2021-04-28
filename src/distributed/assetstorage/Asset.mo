import HashMap "mo:base/HashMap";
import Iter "mo:base/Iter";
import Text "mo:base/Text";
import Time "mo:base/Time";

import T "Types";
import U "Utils";

module {
  public type AssetEncoding = {
    // The last modified date. Since we don't reuse asset entries right now,
    // we can't keep the created date.
    modified: Time.Time;
    contentEncoding: Text;
    content: [Blob];
    totalLength: Nat;
    sha256: ?Blob;
  };

  public class Asset(
    initContentType: Text,
    initEncodings: HashMap.HashMap<Text, AssetEncoding>
  ) {
    public let contentType = initContentType;
    let encodings = initEncodings;

    // Naive encoding selection: of the accepted encodings, pick the first available.
    public func chooseEncoding(acceptEncodings : [Text]) : ?AssetEncoding {
      for (acceptEncoding in acceptEncodings.vals()) {
        switch (encodings.get(acceptEncoding)) {
          case null {};
          case (?encoding) return ?encoding;
        }
      };
      null
    };

    public func getEncoding(encodingType: Text): ?AssetEncoding {
      encodings.get(encodingType)
    };

    public func setEncoding(encodingType: Text, encoding: AssetEncoding) {
      encodings.put(encodingType, encoding)
    };

    public func unsetEncoding(encodingType: Text) {
      encodings.delete(encodingType)
    };

    public func encodingEntries() : Iter.Iter<(Text,AssetEncoding)> = encodings.entries();

    public func toStableAsset() : StableAsset = {
      contentType = contentType;
      encodings = Iter.toArray(encodings.entries());
    };
  };

  public type StableAsset = {
    contentType: Text;
    encodings: [(Text, AssetEncoding)];
  };

  public func toStableAssetEntry((k: T.Key, v: Asset)) : ((T.Key, StableAsset)) {
    (k, v.toStableAsset())
  };

  public func toAssetEntry((k: T.Key, v: StableAsset)) : ((T.Key, Asset)) {
    let a = Asset(
      v.contentType,
      HashMap.fromIter(v.encodings.vals(), 7, Text.equal, Text.hash)
    );
    (k, a)
  };
}
