import Cycles "mo:base/ExperimentalCycles";
import Error "mo:base/Error";
import Principal "mo:base/Principal";
import Text "mo:base/Text";
import Debug "mo:base/Debug";


actor class Coupon() = self {
    type Management = actor {
        deposit_cycles : ({canister_id : Principal}) -> async ();
    };
    type CyclesLedger = actor {
        deposit : (DepositArgs) -> async (DepositResult);
    };
    type Account = {
        owner : Principal;
        subaccount : ?Blob;
    };
    type DepositArgs = {
        to : Account;
        memo : ?Blob;
    };
    type DepositResult = { balance : Nat; block_index : Nat };


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

    // Redeem coupon code to cycle ledger
    public shared (args) func redeem_to_cycles_ledger(code: Text, account: Account) : async DepositResult {
        if (code == "invalid") {
            throw(Error.reject("Code is expired or not redeemable"));
        };
        let CyclesLedgerCanister : CyclesLedger = actor("um5iw-rqaaa-aaaaq-qaaba-cai");
        var amount = 10000000000000;
        Cycles.add(amount);
        let result = await CyclesLedgerCanister.deposit({
                to = account;
                memo = null
        });
        return result;
    };
};
