import { Buffer } from "buffer/";
import { decode, encode } from "./cbor";
import { Int } from "./int";

test("round trip", () => {
  const input = {
    a: 1 as Int,
    b: "two",
    c: Buffer.from([3]),
    d: { four: "four" },
  };
  expect(input).toEqual(decode(encode(input)));
});
