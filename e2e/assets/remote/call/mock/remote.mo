actor {
  var id : Text = "";
    
  public func write(v: Text) : async () {
    id := v;
  };
  public query func read() : async Text {
    id
  };

  public query func actual_query_mock_query_remote_candid_query(v: Text) : async Text {
    id := v; // no effect if called as a query; makes a change if called as an update
    v # " actual actual_query_mock_query_remote_candid_query"
  };

  public func actual_query_mock_update_remote_candid_query(v: Text) : async () {
    id := v;
  };
  public query func actual_update_mock_query_remote_candid_update(v: Text) : async Text {
    v # " mock actual_update_mock_query_remote_candid_update"
  };
  public func actual_query_mock_query_remote_candid_update(v: Text) : async Text {
    id := v;
    v # " actual actual_query_mock_query_remote_candid_update"
  };

  public func actual_update_remote_candid_query(v: Text) : async () {
    id := v;
  };

  // Subtle: since the code is different, the module hashes are different.
  // This makes it so tests can detect if an upgrade is attempted,
  // since install/deploy skip upgrades if the module hashes are the same.
  public query func which_am_i() : async Text {
    "mock"
  };
};
