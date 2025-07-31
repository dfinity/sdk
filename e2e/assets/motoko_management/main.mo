import Management "ic:aaaaa-aa";

persistent actor {
  public func rand() : async Blob {
    await Management.raw_rand();
  };
};
