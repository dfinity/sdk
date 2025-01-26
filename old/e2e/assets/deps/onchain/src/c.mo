import a "canister:a";

actor class c(num : ?Nat) {
    stable var NUM : Nat = switch (num) {
        case (?n) { n };
        case (null) { 3 };
    };

    public query func get() : async Nat {
        return NUM;
    };

    public func times_a() : async Nat {
        let res = NUM * (await a.get());
        return res;
    };
};
