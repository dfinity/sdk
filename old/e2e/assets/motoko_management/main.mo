import Management "ic:aaaaa-aa";

actor {
  public func rand() : async Blob {
    await Management.raw_rand();
  };
};
