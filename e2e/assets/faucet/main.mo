import Cycles "mo:base/ExperimentalCycles";
import Error "mo:base/Error";
import Principal "mo:base/Principal";
import Text "mo:base/Text";

actor class Coupon() = self {
    type Management = actor {
      deposit_cycles : ({canister_id : Principal}) -> async ();
    };

    // Uploading wasm is hard. This is much easier to handle.
    var wallet_to_hand_out: ?Principal = null;
    public func set_wallet_to_hand_out(wallet: Principal) : async () {
        wallet_to_hand_out := ?wallet;
    };

    // Redeem coupon code to create a cycle wallet
    public shared (args) func redeem(code: Text) : async Principal {
        if (code == "invalid") {
            throw(Error.reject("Code is expired or not redeemable"));
        };
        switch (wallet_to_hand_out) {
            case (?wallet) {
                return wallet;
            };
            case (_) {
                throw(Error.reject("Set wallet to return before calling this!"));
            };
        };
    };

    // Redeem coupon code to top up an existing wallet
    public func redeem_to_wallet(code: Text, wallet: Principal) : async Nat {
        if (code == "invalid") {
            throw(Error.reject("Code is expired or not redeemable"));
        };
        let IC0 : Management = actor("aaaaa-aa");
        var amount = 10000000000000;
        Cycles.add(amount);
        await IC0.deposit_cycles({ canister_id = wallet });
        return amount;
    };
};
