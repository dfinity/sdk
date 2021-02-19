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
  public var table : [var KVs<K,V>] = [var];
  public var _count : Nat = 0;
};

public class StableHashMapManipulator<K, V>(
  initCapacity: Nat,
  keyEq: (K,K) -> Bool,
  keyHash: K -> Hash.Hash
) {

  /// Gets the entry with the key `k` and returns its associated value if it
  /// existed or `null` otherwise.
  public func get(shm: StableHashMap<K, V>, k:K) : ?V {
    let h = Prim.word32ToNat(keyHash(k));
    let m = shm.table.size();
    let v = if (m > 0) {
      AssocList.find<K,V>(shm.table[h % m], k, keyEq)
    } else {
      null
    };
  };

  /// Insert the value `v` at key `k`. Overwrites an existing entry with key `k`
  public func put(m: StableHashMap<K, V>, k : K, v : V) = ignore replace(m, k, v);

  /// Insert the value `v` at key `k` and returns the previous value stored at
  /// `k` or null if it didn't exist.
  public func replace(m: StableHashMap<K, V>, k:K, v:V) : ?V {
    if (m._count >= m.table.size()) {
      let size =
        if (m._count == 0) {
          if (initCapacity > 0) {
            initCapacity
          } else {
            1
          }
        } else {
          m.table.size() * 2;
        };
      let table2 = A.init<KVs<K,V>>(size, null);
      for (i in m.table.keys()) {
        var kvs = m.table[i];
        label moveKeyVals : ()
        loop {
          switch kvs {
          case null { break moveKeyVals };
          case (?((k, v), kvsTail)) {
                 let h = Prim.word32ToNat(keyHash(k));
                 let pos2 = h % table2.size();
                 table2[pos2] := ?((k,v), table2[pos2]);
                 kvs := kvsTail;
               };
          }
        };
      };
      m.table := table2;
    };
    let h = Prim.word32ToNat(keyHash(k));
    let pos = h % m.table.size();
    let (kvs2, ov) = AssocList.replace<K,V>(m.table[pos], k, keyEq, ?v);
    m.table[pos] := kvs2;
    switch(ov){
    case null { m._count += 1 };
    case _ {}
    };
    ov
  };

};

}