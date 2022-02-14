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

  type Phone = Text;
  type Struct = {
    a: Text;
    b: Phone;
  };

  public query func make_struct(a: Text, b: Text) : async Struct {
    let result = { a = a; b = b; };
    result
  };

  public query func actual_query_mock_query_remote_candid_query(v: Text) : async Text {
    id := v; // no effect if called as a query; makes a change if called as an update
    v # " actual actual_query_mock_query_remote_candid_query"
  };

  public query func actual_query_mock_update_remote_candid_query(v: Text) : async () {
    id := v;
  };
  public func actual_update_mock_query_remote_candid_update(v: Text) : async Text {
        v # " actual actual_update_mock_query_remote_candid_update"
  };

  public query func actual_query_mock_query_remote_candid_update(v: Text) : async Text {
    id := v; // no effect if called as a query; makes a change if called as an update
    v # " actual actual_query_mock_query_remote_candid_update"
  };

  public func actual_update_remote_candid_query(v: Text) : async () {
    id := v;
  };
};
