actor class C(p: { x: Nat; y: Int }) {
  type Profile = { name: Principal; kind: {#admin; #user; #guest }; age: ?Nat8; };
  type List = (Profile, ?List);
  public query func echo(x: List) : async List {
    x
  }
};
