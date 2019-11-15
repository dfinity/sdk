// tslint:disable-next-line: max-line-length
// https://github.com/dfinity-lab/dfinity/blob/9bca65f8edd65701ea6bdb00e0752f9186bbc893/docs/spec/public/index.adoc#cbor-encoding-of-requests-and-responses
import BigNumber from "bignumber.js";
import borc from "borc";
import { BinaryBlob } from "./blob";
import { CanisterId } from "./canisterId";

interface CborRecord extends Record<string, CborValue> {}

export type CborValue
  // Strings: Major type 3 (“Text string”).
  = string

  // Blobs: Major type 2 (“Byte string”)
  | BinaryBlob

  // Integer numbers: Major type 0 or 1 (“Unsigned/signed integer”) if small
  // enough to fit that type, else the Bignum format is used.
  | BigNumber

  // Nested records: Major type 5 followed by string keys.
  | CborRecord

  // Canister IDs are currently represented as u64 in the HTTP handler of the
  // client.
  | CanisterId;

export enum CborTag {
  Uint64LittleEndian = 71,
  Semantic = 55799,
}

export const encode = (value: CborValue): BinaryBlob => {
  const buffer = borc.encode(
    new borc.Tagged(CborTag.Semantic, value),
  );
  return new Uint8Array(buffer) as BinaryBlob;
};

export const decode = (input: Uint8Array): CborValue => {
  const decoder = new borc.Decoder({
    size: input.byteLength,
    tags: {
      [CborTag.Semantic]: (value: CborValue): CborValue => value,
    },
  });
  return decoder.decodeFirst(input);
};
