import Prim "mo:prim";

actor {
    stable var version = 0;

    version += 1;
    Prim.debugPrint("Deployed actor version " # debug_show (version));

    public query func getVersion() : async Nat {
        return version;
    };
};
