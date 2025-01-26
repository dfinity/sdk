actor {
  stable var state : Int = 0;
  public func inc() : async Int {
    state += 1;
    return state;
  };
  public func f() : async ?Int {
    return ?42;
  };
  public query func read() : async Int { return state; };
}

