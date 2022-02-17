actor {
  var id : Text = "";
    
  public func write(v: Text) : async () {
    id := v;
  };
  public query func read() : async Text {
    id
  };

  public query func actual_update_mock_query_remote_candid_update(v: Text) : async Text {
    v # " mock actual_update_mock_query_remote_candid_update"
  };

  // Subtle: since the code is different, the module hashes are different.
  // This makes it so tests can detect if an upgrade is attempted,
  // since install/deploy skip upgrades if the module hashes are the same.
  public query func which_am_i() : async Text {
    "mock"
  };
};
