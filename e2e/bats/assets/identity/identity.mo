import P "mo:base/Principal";

actor Self {
    public shared(msg) func fromCall(): async Principal {
        msg.caller
    };
    public shared query(msg) func fromQuery() : async Principal {
        msg.caller
    };
    public query func getCanisterId() : async Principal {
        P.fromActor(Self)
    };
    public query func isMyself(id: Principal) : async Bool {
        id == P.fromActor(Self)
    };    
};
