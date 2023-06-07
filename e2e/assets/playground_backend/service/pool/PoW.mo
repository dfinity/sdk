import Splay "mo:splay";
import Time "mo:base/Time";
import Text "mo:base/Text";
import Int "mo:base/Int";
import Debug "mo:base/Debug"

module {
    public type Nonce = {
        timestamp : Int;
        nonce : Nat;
    };
    func nonceCompare(x : Nonce, y : Nonce) : { #less; #equal; #greater } {
        if (x.timestamp < y.timestamp) { #less } else if (x.timestamp == y.timestamp and x.nonce < y.nonce) {
            #less;
        } else if (x.timestamp == y.timestamp and x.nonce == y.nonce) { #equal } else {
            #greater;
        };
    };
    public class NonceCache(TTL : Nat) {
        let known_nonces = Splay.Splay<Nonce>(nonceCompare);
        public func add(nonce : Nonce) {
            known_nonces.insert(nonce);
        };
        public func pruneExpired() {
            let now = Time.now();
            for (info in known_nonces.entries()) {
                if (info.timestamp > now - TTL) { return };
                known_nonces.remove(info);
            };
        };
        public func contains(nonce : Nonce) : Bool {
            known_nonces.find(nonce);
        };
        public func checkProofOfWork(nonce : Nonce) : Bool {
            let now = Time.now();
            if (nonce.timestamp < now - TTL) {
                Debug.trap("too late");
                return false;
            };
            if (nonce.timestamp > now + TTL) {
                Debug.trap("too early");
                return false;
            };
            let raw = "motoko-playground" # (Int.toText(nonce.timestamp)) # (Int.toText(nonce.nonce));
            Debug.print(raw);
            let hash = Text.hash(raw);
            Debug.print("The Motoko-calculated hash is " # debug_show (hash));
            if (hash & 0xc0000000 != 0) {
                Debug.trap("other stuff failed");
                return false;
            };
            true;
        };
    };
};
