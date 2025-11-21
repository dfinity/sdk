import Rand "dependency";

persistent actor {
  public func rand() : async Blob {
    await Rand.rand();
  };
};
