actor {
  public query func greet(name : Text) : async Text {
    return "Hello, " # name # "!";
  };

  stable var a : Nat = 0;
  public func inc_a() : async Nat {
    a += 1;
    return a;
  };

  stable var b : Int = 0;
  public func inc_b() : async Int {
    b += 1;
    return b;
  };
};
