type List<T> = ?{head : T; tail : List<T>};

actor {
    //TODO Bug fix for inline polymorphic type
    //public type List<T> = ?{head : T; tail : List<T>};
    func map(l: List<Int>) : List<Int> = {
        switch l {
          case null { null };
          case (?v) { ?{head=v.head+1; tail=map(v.tail)} };
        }
    };
    public func inc(i: Int, b: Bool, str: Text, vec: [Nat], l: List<Int>) : async (Int, Bool, Text, [Nat], List<Int>) {
        let arr = Array_tabulate<Nat>(
          vec.len(),
          func (i : Nat) : Nat {
              vec[i]+1;
          });

        var text = "";
        for (c in str.chars()) {
            let c2 = word32ToChar(charToWord32(c)+1);
            text := text # charToText(c2);
        };
        return (i+1, not b, text, arr, map(l));
    };
};

