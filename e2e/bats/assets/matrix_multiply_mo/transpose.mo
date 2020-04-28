import A "mo:stdlib/Array";

type Matrix = [[Int]];

actor {
    public query func transpose(m: Matrix) : async Matrix {
        assert (m.len() > 0);
        let n_row = m.len();
        let n_col = m[0].len();
        A.tabulate<[Int]>(
          n_col,
          func (j:Nat) : [Int] {
              A.tabulate<Int>(n_row, func (i:Nat) : Int = (m[i][j]));
          });
    };
};
