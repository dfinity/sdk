actor {
    type Vec = [Int];
    var vec : Vec = [];
    
    public func init(a: Vec) : async () {
        vec := a;
    };
    public query func dot_product_with(b: Vec) : async Int {
        assert (vec.size() == b.size());
        var res: Int = 0;
        let len = vec.size();
        var i = 0;
        while (i < len) {
            res := res + vec[i]*b[i];
            i += 1;
        };
        res
    };
};
