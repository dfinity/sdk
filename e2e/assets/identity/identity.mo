import P "mo:base/Principal";

shared {caller} actor class () = Self {
    private let initializer : Principal = caller;

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

    public shared query(msg) func amInitializer() : async Bool {
        msg.caller == initializer
    };
};
