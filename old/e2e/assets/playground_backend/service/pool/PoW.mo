import Splay "mo:splay";
import Time "mo:base/Time";
import Text "mo:base/Text";
import Int "mo:base/Int";

module {
    public type Nonce = {
        timestamp: Int;
        nonce: Nat;
    };
    func nonceCompare(x: Nonce, y: Nonce): {#less;#equal;#greater} {
        if (x.timestamp < y.timestamp) { #less }
        else if (x.timestamp == y.timestamp and x.nonce < y.nonce) { #less }
        else if (x.timestamp == y.timestamp and x.nonce == y.nonce) { #equal }
        else { #greater }
    };
    public class NonceCache(TTL: Nat) {
        let known_nonces = Splay.Splay<Nonce>(nonceCompare);
        public func add(nonce: Nonce) {
            known_nonces.insert(nonce);
        };
        public func pruneExpired() {
            let now = Time.now();
            for (info in known_nonces.entries()) {
                if (info.timestamp > now - TTL) { return; };
                known_nonces.remove(info);
            };
        };
        public func contains(nonce: Nonce) : Bool {
            known_nonces.find(nonce)
        };
        public func checkProofOfWork(nonce: Nonce) : Bool {
            let now = Time.now();
            if (nonce.timestamp < now - TTL) return false;
            if (nonce.timestamp > now + TTL) return false;
            let raw = "motoko-playground" # (Int.toText(nonce.timestamp)) # (Int.toText(nonce.nonce));
            let hash = Text.hash(raw);
            if (hash & 0xc0000000 != 0) return false;
            true
        };
    };
}
