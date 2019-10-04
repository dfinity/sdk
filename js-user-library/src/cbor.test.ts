import { Buffer } from "buffer/";
import { toHex } from "./buffer";
import { CborValue, decode, encode } from "./cbor";
import { Int } from "./int";

test("round trip", () => {
  interface Data extends Record<string, CborValue> {
    a: Int;
    b: string;
    c: Buffer;
    d: { four: string };
  }

  const input: Data = {
    a: 1 as Int,
    b: "two",
    c: Buffer.from([3]),
    d: { four: "four" },
  };

  const output = decode(encode(input)) as Data;

  const { c: inputC, ...inputRest } = input;
  const { c: outputC, ...outputRest } = output;
  expect(inputRest).toEqual(outputRest);
  expect(toHex(inputC)).toBe(toHex(outputC));
});
