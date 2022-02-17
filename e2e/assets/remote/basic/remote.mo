actor {
  var id : Text = "";
    
  public func write(v: Text) : async () {
    id := v;
  };
  public query func read() : async Text {
    id
  };

  public query func write_update_on_actual(v: Text) : async () {
    id := v;
  };

  // Subtle: since the code is different, the module hashes are different.
  // This makes it so tests can detect if an upgrade is attempted,
  // since install/deploy skip upgrades if the module hashes are the same.
  public query func which_am_i() : async Text {
    "mock"
  };
};
