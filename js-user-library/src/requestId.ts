import BigNumber from "bignumber.js";
import borc from "borc";
import { Buffer } from "buffer/";
import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { CborValue } from "./cbor";
import { Hex } from "./hex";
import { Int } from "./int";
import * as int from "./int";
import * as Request from "./request";

export type RequestId = BinaryBlob & { __requestId__: void };

export const toHex = (requestId: RequestId): Hex => blob.toHex(requestId);

// The spec describes encoding for these types.
type HashableValue = string | Buffer | Int | BigNumber | borc.Tagged;

export const hash = async (data: BinaryBlob): Promise<BinaryBlob> => {
  const hashed: ArrayBuffer = await crypto.subtle.digest({
    name: "SHA-256",
  }, data.buffer);
  return Buffer.from(hashed) as BinaryBlob;
};

const padHex = (hex: Hex): Hex => {
  return `${"0000000000000000".slice(hex.length)}${hex}` as Hex;
};

const hashValue = (value: HashableValue): Promise<Buffer> => {
  if (isTagged(value)) {
    return hashValue(value.value);
  } else if (isString(value)) {
    return hashString(value as string);
  } else if (isBigNumber(value)) {
    // HTTP handler expects canister_id to be an u64 & hashed in this way.
    const hex = value.toString(16) as Hex;
    const padded = padHex(hex);
    return hash(blob.fromHex(padded));
  } else if (isInt(value)) {
    const hex = int.toHex(value);
    const padded = padHex(hex);
    return hash(blob.fromHex(padded));
  } else if (isBlob(value)) {
    return hash(value as BinaryBlob);
  } else {
    throw new Error(`Attempt to hash a value of unsupported type: ${value}`);
  }
};

const hashString = (value: string): Promise<BinaryBlob> => {
  const encoder = new TextEncoder();
  const encoded: Uint8Array = encoder.encode(value);
  return hash(Buffer.from(encoded) as BinaryBlob);
};

const isBigNumber = (value: HashableValue): value is BigNumber => {
  return value instanceof BigNumber;
};

const isBlob = (value: HashableValue): value is BinaryBlob => {
  return value instanceof Buffer;
};

const isInt = (value: HashableValue): value is Int => {
  return typeof value === "number";
};

const isString = (value: HashableValue): value is string => {
  return typeof value === "string";
};

const isTagged = (value: HashableValue): value is borc.Tagged => {
  return value instanceof borc.Tagged;
};

const concat = (bs: Array<BinaryBlob>): BinaryBlob => {
  return bs.reduce((state: Uint8Array, b: BinaryBlob): Uint8Array => {
    return new Uint8Array([ ...state, ...b ]);
  }, new Uint8Array()) as BinaryBlob;
};

export const requestIdOf = async (
  request: Request.CommonFields,
): Promise<RequestId> => {
  // While the type signature of this function ensures the fields we care about
  // are present, it does not prevent additional fields from being provided,
  // including the fields used for authentication that we must omit when
  // calculating the request ID. This is by design, since requests are expected
  // to have more than just the common fields. As a result, we need to explictly
  // ignore the authentication fields.
  const { sender_pubkey, sender_sig, ...fields } = request;

  const hashed: Array<Promise<[BinaryBlob, BinaryBlob]>> = Object
    .entries(fields)
    .map(async ([key, value]: [string, CborValue]) => {
      const hashedKey = await hashString(key);
      const hashedValue = await hashValue(value as HashableValue);

      return [
        hashedKey,
        hashedValue,
      ] as [BinaryBlob, BinaryBlob];
    });

  const traversed: Array<[BinaryBlob, BinaryBlob]> = await Promise.all(hashed);

  const sorted: Array<[BinaryBlob, BinaryBlob]> = traversed
    .sort(([k1, v1], [k2, v2]) => {
      return Buffer.compare(Buffer.from(k1), Buffer.from(k2));
    });

  const concatenated: BinaryBlob = concat(sorted.map(concat));
  const requestId = await hash(concatenated) as RequestId;
  return requestId;
};
