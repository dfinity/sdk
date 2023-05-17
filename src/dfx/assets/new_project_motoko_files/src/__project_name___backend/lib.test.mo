import { test; suite } "mo:test";
import Lib "./lib";

suite(
  "Methods",
  func() : () {
    test(
      "greet",
      func() {
        let result = Lib.greet("developer");
        assert (result == "Hello, developer!");
      },
    );
  },
);
