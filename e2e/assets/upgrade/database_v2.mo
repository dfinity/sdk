actor {
  type Data = { name: Text; age: ?Nat };

  stable var db: [Data] = [];

  public func add() : async () {
      db := [ { name = "test"; age = ?42 } ];
  };

  public query func dump() : async Text {
      debug_show db
  };
};

