actor Call {

  public query func make_struct(a: Text, b: Text) : async { c: Text; d: Text; } {
    let result = { c = a; d = b; };
    result
  };

}
