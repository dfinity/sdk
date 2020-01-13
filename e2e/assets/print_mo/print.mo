import Debug "mo:stdlib/debug.mo";

actor HelloActor {
  public func hello() : async () {
    Debug.print("Hello, World! from DFINITY \n");
  }
};
