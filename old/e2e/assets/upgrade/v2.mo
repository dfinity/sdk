import Int "mo:base/Int";
actor {
  stable var state : Int = 0;
  stable var new_state : Nat = Int.abs(state);
  public func inc() : async Nat {
    new_state += 1;
    return new_state;
  };
  public func f() : async ?Int {
    return ?42;
  };
  public query func read() : async Nat { return new_state; };
}

