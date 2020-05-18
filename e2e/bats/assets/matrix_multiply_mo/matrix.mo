import A "mo:base/Array";
import M "secret_import";

type Matrix = [[Int]];

actor {
    public func multiply(a: Matrix, b: Matrix) : async Matrix {
        assert (a.len() > 0 and b.len() > 0);
        assert (a[0].len() == b.len());
        let n = a.len();
        let k = b[0].len();
        let bt = await M.T.transpose(b);
        let res : [[var Int]] = A.tabulate<[var Int]>(n, func (_:Nat):[var Int] = A.init<Int>(k, 0));
        var i = 0;
        while (i < n) {
            await M.D.init(a[i]);
            var j = 0;
            while (j < k) {
                res[i][j] := await M.D.dot_product_with(bt[j]);
                j += 1;
            };
            i += 1;
        };
        A.tabulate<[Int]>(
          n,
          func (i:Nat) : [Int] { A.freeze<Int>(res[i]) });
    };
};
