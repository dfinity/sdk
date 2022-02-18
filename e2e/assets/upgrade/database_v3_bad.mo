actor {
  type Data = { name: Text; age: ?Nat };

  stable var db: [Data] = [];

  public func add2() : async () {
      db := [ { name = "test"; age = ?42 } ];
  };

  public query func dump() : async Text {
      debug_show db
  };
};

