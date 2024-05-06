import Management "ic:aaaaa-aa";

module Rand {
  public func rand() : async Blob {
    await Management.raw_rand();
  };
};
