import H "mo:base/HashMap";
import Iter "mo:base/Iter";

module Utils {

  public func clearHashMap<K, V>(h:H.HashMap<K,V>) : () {
    let keys = Iter.toArray(Iter.map(h.entries(), func((k: K, v: V)): K { k }));
    for (key in keys.vals()) {
      h.delete(key);
    };
  };
};