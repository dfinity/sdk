import Prim "mo:prim";
import P "mo:base/Principal";

actor {
    public shared(msg) func hashFromCall(): async Nat {
        Prim.word32ToNat(P.hash(msg.caller))
    };
    public shared query(msg) func hashFromQuery() : async Nat {
        Prim.word32ToNat(P.hash(msg.caller))
    };
};
