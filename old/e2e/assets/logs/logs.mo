import Debug "mo:base/Debug";

actor HelloActor {
  public func hello(name : Text) : async () {
    Debug.print("Hello, " # name # "!\n");
  };
};
