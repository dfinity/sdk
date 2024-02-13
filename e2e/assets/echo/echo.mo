actor {
  type Profile = { name: Text; kind: {#admin; #user; #guest }; age: ?Nat8; };
  type List = (Profile, ?List);
  public query func echo(x: List) : async List {
    x
  }
};
