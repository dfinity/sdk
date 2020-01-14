type Vec = [Int];

actor {
    var vec : Vec = [];
    
    public func init(a: Vec) : async () {
        vec := a;
    };
    public query func dot_product_with(b: Vec) : async Int {
        assert (vec.len() == b.len());
        var res: Int = 0;
        let len = vec.len();
        var i = 0;
        while (i < len) {
            res := res + vec[i]*b[i];
            i += 1;
        };
        res
    };
};
