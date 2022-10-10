actor {
  var id : Text = "";

  //
  public func something_extra() : async Text {
    "extra!"
  };
    
  public func write(v: Text) : async () {
    id := v;
  };
  public query func read() : async Text {
    id
  };

  // Subtle: since the code is different, the module hashes are different.
  // This makes it so tests can detect if an upgrade is attempted,
  // since the module hashes would be different.
  public query func which_am_i() : async Text {
    "mock"
  };
};
