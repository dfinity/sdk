import A "mo:base/Array";
import Prim "mo:prim";

type List<T> = ?{head : T; tail : List<T>};
type List2<T> = { #nil; #cons: (T, List2<T>) };

actor {
    func map(l: List<Int>) : List<Int> = {
        switch l {
          case null { null };
          case (?v) { ?{head=v.head+1; tail=map(v.tail)} };
        }
    };
    func map2(l: List2<Int>) : List2<Int> = {
         switch l {
           case (#nil) { #nil };
           case (#cons(h, tl)) { #cons(h+1, map2 tl) };
         }
    };
    public func inc(i: Int, b: Bool, str: Text, vec: [Nat], l: List<Int>, l2: List2<Int>) : async (Int, Bool, Text, [Nat], List<Int>, List2<Int>) {
        let arr = A.tabulate<Nat>(
          vec.size(),
          func (i : Nat) : Nat {
              vec[i]+1;
          });

        var text = "";
        for (c in str.chars()) {
            let c2 = Prim.word32ToChar(Prim.charToWord32(c)+1);
            text := text # Prim.charToText(c2);
        };
        return (i+1, not b, text, arr, map(l), map2(l2));
    };
};
