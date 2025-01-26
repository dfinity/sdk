actor {
  stable var state : Int = 0;
  public func inc2() : async Int {
    state += 1;
    return state;
  };
  public func f() : async ?Int {
    return ?42;
  };
  public query func read2() : async Int { return state; };
}

