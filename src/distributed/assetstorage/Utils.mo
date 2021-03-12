import Hash "mo:base/Hash";
import HashMap "mo:base/HashMap";
import Int "mo:base/Int";
import Iter "mo:base/Iter";

module Utils {

  public func clearHashMap<K, V>(h:HashMap.HashMap<K, V>) : () {
    let keys = Iter.toArray(Iter.map(h.entries(), func((k: K, _: V)): K = k ));
    for (key in keys.vals()) {
      h.delete(key);
    };
  };

  public func deleteFromHashMap<K, V>
    (h:HashMap.HashMap<K,V>,
     keyEq: (K,K) -> Bool,
     keyHash: K -> Hash.Hash,
     deleteFn: (K, V) -> Bool
    ): () {
    let entriesToDelete: HashMap.HashMap<K, V> = HashMap.mapFilter<K, V, V>(h, keyEq, keyHash,
      func(k: K, v: V) : ?V {
        if (deleteFn(k, v))
          ?v
        else
          null
      }
    );
    for ((k,_) in entriesToDelete.entries()) {
      h.delete(k);
    };
  }
};
