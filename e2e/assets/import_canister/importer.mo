import Hello "canister:origin";

persistent actor {
    public func greet_piped(name : Text) : async Text {
        let response = await Hello.greet(name);
        return response;
    };
};
