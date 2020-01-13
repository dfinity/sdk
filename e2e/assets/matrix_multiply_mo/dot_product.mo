type Vec = [Int];

actor {
    public func dot_product(a:Vec, b:Vec) : async Int {
        assert (a.len() == b.len());
        var res: Int = 0;
        let len = a.len();
        var i = 0;
        while (i < len) {
            res := res + a[i]*b[i];
            i += 1;
        };
        res
    };
};
