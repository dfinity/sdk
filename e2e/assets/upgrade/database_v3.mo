actor {
  type Data = { name: Text; age: ?Nat };

  stable var db: [Data] = [];

  public func add() : async Text {
      db := [ { name = "test"; age = ?42 } ];
      "ok"
  };

  public query func dump() : async Text {
      debug_show db
  };
};

