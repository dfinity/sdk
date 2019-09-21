import { decode, encode } from "./cbor";
import { Int } from "./int";

test("round trip", () => {
  const input = {
    a: 1 as Int,
    b: "two",
    c: [ 3 as Int ],
    d: { four: "four" },
  };
  expect(decode(encode(input))).toEqual(input);
});
