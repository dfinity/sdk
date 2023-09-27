import Rand "dependency";

actor {
  public func rand() : async Blob {
    await (await Rand.rand());
  };
};
