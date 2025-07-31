import R "canister:remote";

persistent actor {
    public func read_remote() : async Text {
        await R.read()
    };

    public func remote_extra() : async Text {
        await R.something_extra()
    };
};
