// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  public type Config = {
    backend_canister_id : ?Principal;
    remove_cycles_add : Bool;
    profiling : ?{ start_page : ?Nat32; page_limit : ?Nat32 };
    limit_stable_memory_page : ?Nat32;
    limit_heap_memory_page : ?Nat32;
  };
  public type Self = actor {
    is_whitelisted : shared query Blob -> async Blob;
    transform : shared query (Blob, Config) -> async Blob;
  }
}