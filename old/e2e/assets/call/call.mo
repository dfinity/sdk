actor {
  public query func make_struct(a: Text, b: Text) : async { c: Text; d: Text; } {
    let result = { c = a; d = b; };
    result
  };
  public query func make_struct2(a: Text, b: Text) : async { c: Text; d: Text; } {
    let result = { c = a; d = b; };
    result
  };
};
