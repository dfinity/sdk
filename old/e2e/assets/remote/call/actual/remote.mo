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

  public query func make_struct(a: Text, b: Text) : async { a: Text; b: Text; } {
    let result = { a = a; b = b; };
    result
  };

  public func actual_update_mock_query_remote_candid_update(v: Text) : async Text {
        v # " actual actual_update_mock_query_remote_candid_update"
  };
};
