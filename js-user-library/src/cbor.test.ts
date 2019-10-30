import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { CborValue, decode, encode } from "./cbor";
import { Int } from "./int";

test("round trip", () => {
  interface Data extends Record<string, CborValue> {
    a: Int;
    b: string;
    c: BinaryBlob;
    d: { four: string };
  }

  const input: Data = {
    a: 1 as Int,
    b: "two",
    c: Uint8Array.from([3]) as BinaryBlob,
    d: { four: "four" },
  };

  const output = decode(encode(input)) as Data;

  // A `Uint8Array` value doesn't decode exactly to the value that was encoded,
  // but their hexadecimal representions are the same.
  const { c: inputC, ...inputRest } = input;
  const { c: outputC, ...outputRest } = output;
  expect(outputRest).toEqual(inputRest);
  expect(blob.toHex(outputC)).toBe(blob.toHex(inputC));
});
