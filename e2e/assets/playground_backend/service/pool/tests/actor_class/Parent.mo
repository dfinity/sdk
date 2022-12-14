import Array "mo:base/Array";
import Cycles "mo:base/ExperimentalCycles";
import Child "Child";
import Principal "mo:base/Principal";

actor Parent {
  type canister_settings = {
    controllers : ?[Principal];
    freezing_threshold : ?Nat;
    memory_allocation : ?Nat;
    compute_allocation : ?Nat;
  };

  let IC =
    actor "aaaaa-aa" : actor {
      create_canister : { } -> async { canister_id : Principal };
      stop_canister : { canister_id : Principal } -> async ();
      start_canister : { canister_id : Principal } -> async ();
      delete_canister : { canister_id : Principal } -> async ();
      update_settings : shared {
          canister_id : Principal;
          settings : canister_settings;
        } -> async ();
    };

  type Child = Child.Child;
  let children : [var ?Child] = Array.init(5, null);

  public func sayHi(i : Nat) : async ?Text {
    do ? {
      await children[i]!.sayHi()
    }
  };

  public func makeChild(i : Nat) : async Principal {
    Cycles.add 550_000_000_000;
    let b = await Child.Child();
    children[i] := ?b;
    Principal.fromActor b
  };

  public func deleteChild(i : Nat) : async () {
    ignore do ? {
      await IC.delete_canister { canister_id = Principal.fromActor(children[i]!) };
      children[i] := null;
    }
  };

  public func stopChild(i : Nat) : async () {
    ignore do ? {
      await IC.stop_canister { canister_id = Principal.fromActor(children[i]!) };
    }
  };

  public func startChild(i : Nat) : async () {
    ignore do ? {
      await IC.start_canister { canister_id = Principal.fromActor(children[i]!) };
    }
  };

  public func updateChildSettings(i : Nat, settings : canister_settings) : async () {
    ignore do ? {
      await IC.update_settings { canister_id = Principal.fromActor(children[i]!); settings };
    }
  }
}