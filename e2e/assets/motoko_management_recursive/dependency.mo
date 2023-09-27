import Management "ic:aaaaa-aa";

actor class Rand {
  public func rand() : async Blob {
    await Management.raw_rand();
  };
};
