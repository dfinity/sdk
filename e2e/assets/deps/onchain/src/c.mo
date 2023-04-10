actor class c (num : Nat) {
    stable var NUM : Nat = num;

    public query func get() : async Nat {
        return NUM;
    };
};
