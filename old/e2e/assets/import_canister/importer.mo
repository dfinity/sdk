import Hello "canister:origin";

actor {
    public func greet_piped(name : Text) : async Text {
        let response = await Hello.greet(name);
        return response;
    };
};
