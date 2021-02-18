import Prim "mo:prim";
import P "mo:base/Prelude";
import A "mo:base/Array";
import Hash "mo:base/Hash";
import Iter "mo:base/Iter";
import AssocList "mo:base/AssocList";

module {

// key-val list type
type KVs<K,V> = AssocList.AssocList<K,V>;

public class StableHashMap<K,V>() {
  var table : [var KVs<K,V>] = [var];
  var _count : Nat = 0;
};

}