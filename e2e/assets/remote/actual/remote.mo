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
};
