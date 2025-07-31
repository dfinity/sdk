import Debug "mo:base/Debug";

persistent actor HelloActor {
  public func hello() : async () {
    Debug.print("Hello, World! from DFINITY \n");
  }
};
