import R "canister:remote";

persistent actor {
    public func read_remote() : async Text {
        await R.read()
    };
};
