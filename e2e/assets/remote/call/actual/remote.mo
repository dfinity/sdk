actor {
  var id : Text = "";
    
  public func write(v: Text) : async () {
    id := v;
  };
  public query func read() : async Text {
    id
  };

  public query func which_am_i() : async Text {
    "actual"
  };

  public query func actual_query_mock_update_remote_candid_query(v: Text) : async () {
    id := v;
  };
  public func actual_update_mock_query_remote_candid_update(v: Text) : async Text {
        v # " actual actual_update_mock_query_remote_candid_update"
  };

  public func actual_update_remote_candid_query(v: Text) : async () {
    id := v;
  };
};
