actor {
  stable var state : Nat = 0;
  public func inc() : async Nat {
    state += 1;
    return state;
  };
  public func f() : async ?Int {
    return ?42;
  };
  public query func read() : async Nat { return state; };
}

