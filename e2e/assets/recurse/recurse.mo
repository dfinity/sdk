actor {
    public func recurse(n : Nat) : async () {
      if (n <= 0) () else await recurse(n - 1);
    };
};
