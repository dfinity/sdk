import R "canister:remote";

actor {
    public func read_remote() : async Text {
        await R.read()
    };

    public func remote_extra() : async Text {
        await R.something_extra()
    };
};
