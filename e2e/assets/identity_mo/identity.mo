actor Identity {
    public shared(ctx) func get_id() : async [Word32] {
        ctx.caller
    };
}
