import BigNumber from "bignumber.js";
import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { CanisterId } from "./canisterId";
import { CborValue, decode, encode } from "./cbor";
import { Hex } from "./hex";

test("round trip", () => {
  interface Data extends Record<string, CborValue> {
    a: BigNumber;
    b: string;
    c: BinaryBlob;
    d: { four: string };
    e: CanisterId;
  }

  const input: Data = {
    a: new BigNumber(1),
    b: "two",
    c: Uint8Array.from([3]) as BinaryBlob,
    d: { four: "four" },
    e: new CanisterId("0000000000000001" as Hex),
  };

  const output = decode(encode(input)) as Data;

  // A `Uint8Array` values don't decode exactly to the value that was encoded,
  // but their hexadecimal representions are the same.
  const { c: inputC, e: inputE, ...inputRest } = input;
  const { c: outputC, e: outputE, ...outputRest } = output;
  expect(outputRest).toEqual(inputRest);
  expect(blob.toHex(outputC)).toBe(blob.toHex(inputC));
  expect(outputE).toBe(inputE.hex); // FIXME: we can't yet decode to CanisterId
});
