import Principal "mo:base/Principal";

actor Parent {
  let IC =
    actor "aaaaa-aa" : actor {
      delete_canister : { canister_id : Principal } -> async ();
    };

  public func deleteCanister(canister_id: Principal) : async () {
      await IC.delete_canister { canister_id };
  };
}