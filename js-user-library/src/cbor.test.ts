import BigNumber from "bignumber.js";
import { Buffer } from "buffer/";
import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { CanisterId } from "./canisterId";
import * as canisterId from "./canisterId";
import { CborValue, decode, encode } from "./cbor";
import { Hex } from "./hex";
import { Int } from "./int";

test("round trip", () => {
  interface Data extends Record<string, CborValue> {
    a: Int;
    b: string;
    c: BinaryBlob;
    d: { four: string };
    e: CanisterId;
    f: BinaryBlob;
    g: BigNumber;
  }

  // FIXME: since we have limited control over CBOR decoding, we are relying on
  // BigNumber types actually containing big numbers, since small numbers are
  // represented as numbers and big numbers are represented as strings.
  const input: Data = {
    a: 1 as Int,
    b: "two",
    c: Buffer.from([3]) as BinaryBlob,
    d: { four: "four" },
    e: canisterId.fromHex("ffffffffffffffff" as Hex),
    f: Buffer.from([]) as BinaryBlob,
    g: new BigNumber("0xffffffffffffffff"),
  };

  const output = decode(encode(input)) as Data;

  // Some values don't decode exactly to the value that was encoded,
  // but their hexadecimal representions are the same.
  const {
    c: inputC,
    f: inputF,
    ...inputRest
  } = input;

  const {
    c: outputC,
    f: outputF,
    ...outputRest
  } = output;

  expect(blob.toHex(outputC)).toBe(blob.toHex(inputC));
  expect(outputRest).toEqual(inputRest);
});
