actor {
  stable var newState : Int = 0;
  public func inc() : async Int {
    newState += 1;
    return newState;
  };
  public func f() : async ?Int {
    return ?42;
  };
  public query func read() : async Int { return newState; };
}
