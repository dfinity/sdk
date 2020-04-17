import Debug "mo:stdlib/Debug";

actor HelloActor {
  public func hello() : async () {
    Debug.print("Hello, World! from DFINITY \n");
  }
};
