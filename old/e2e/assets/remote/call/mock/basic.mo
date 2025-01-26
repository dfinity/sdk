import R "canister:remote";

actor {
    public func read_remote() : async Text {
        await R.read()
    };
};
