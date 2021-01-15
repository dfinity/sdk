import A "mo:base/Array";

actor {
    type Matrix = [[Int]];

    public query func transpose(m: Matrix) : async Matrix {
        assert (m.size() > 0);
        let n_row = m.size();
        let n_col = m[0].size();
        A.tabulate<[Int]>(
          n_col,
          func (j:Nat) : [Int] {
              A.tabulate<Int>(n_row, func (i:Nat) : Int = (m[i][j]));
          });
    };
};
