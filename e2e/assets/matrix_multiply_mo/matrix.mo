import A "mo:stdlib/array.mo";
import D "canister:dot_product";

type Matrix = [[Int]];

actor {
    public func transpose(m: Matrix) : async Matrix {
        assert (m.len() > 0);
        let n_row = m.len();
        let n_col = m[0].len();
        A.tabulate<[Int]>(
          n_col,
          func (j:Nat) : [Int] {
              A.tabulate<Int>(n_row, func (i:Nat) : Int = (m[i][j]));
          });
    };
    public func multiply(a: Matrix, b: Matrix) : async Matrix {
        assert (a.len() > 0 and b.len() > 0);
        assert (a[0].len() == b.len());
        let n = a.len();
        let k = b[0].len();
        let bt = await transpose(b);
        let res : [[var Int]] = A.tabulate<[var Int]>(n, func (_:Nat):[var Int] = A.init<Int>(k, 0));
        var i = 0;
        while (i < n) {
            var j = 0;
            while (j < k) {
                res[i][j] := await D.dot_product(a[i], bt[j]);
                j += 1;
            };
            i += 1;
        };
        A.tabulate<[Int]>(
          n,
          func (i:Nat) : [Int] { A.freeze<Int>(res[i]) });
    };
};
