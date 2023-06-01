import a "canister:a";

actor {
    public query func get() : async Nat {
        return 2;
    };

    public func times_a() : async Nat {
        let res = 2 * (await a.get());
        return res;
    };
};
