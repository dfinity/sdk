actor PrincipalId {
    public shared(ctx) func get_principal_id() : async [Word32] {
        ctx.caller
    };
}
