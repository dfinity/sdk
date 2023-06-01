import dep_b "canister:dep_b";
import dep_c "canister:dep_c";

actor {
    public query func get() : async Nat {
        return 4;
    };

    public func get_b() : async Nat {
        let res = await dep_b.get();
        return res;
    };

    public func get_c() : async Nat {
        let res = await dep_c.get();
        return res;
    };

    public func get_b_times_a() : async Nat {
        let res = await dep_b.times_a();
        return res;
    };

    public func get_c_times_a() : async Nat {
        let res = await dep_c.times_a();
        return res;
    };
};
