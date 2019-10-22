import { Buffer } from "buffer/";
import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { CborValue } from "./cbor";
import { Hex } from "./hex";
import * as int from "./int";
import { Int } from "./int";
import * as Request from "./request";

export type RequestId = BinaryBlob & { __requestId__: void };

export const toHex = (requestId: RequestId): Hex => blob.toHex(requestId);

// The spec describes encoding for these types.
// The exception here is integers, which are used in the current implementation
// of the HTTP handler.
type HashableValue = string | Uint8Array | Int;

export const hash = async (data: Uint8Array): Promise<BinaryBlob> => {
  const hashed = await crypto.subtle.digest({ name: "SHA-256" }, data);
  return new Uint8Array(hashed) as BinaryBlob;
};

const hashValue = (value: HashableValue): Promise<Uint8Array> => {
  if (isString(value)) {
    return hashString(value as string);
  } else if (isInt(value)) {
    // HACK: HTTP handler expects canister_id to be an u64 & hashed in this way.
    const hex = int.toHex(value);
    const padded = `${"0000000000000000".slice(hex.length)}${hex}` as Hex;
    return hash(blob.fromHex(padded));
  } else if (isBlob) {
    return hash(value as BinaryBlob);
  } else {
    throw new Error(`Attempt to hash a value if unsupported type: ${value}`);
  }
};

const hashString = (value: string): Promise<BinaryBlob> => {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(value);
  return hash(encoded);
};

const isBlob = (value: HashableValue): value is BinaryBlob => {
  return value instanceof Uint8Array;
};

const isInt = (value: HashableValue): value is Int => {
  return typeof value === "number";
};

const isString = (value: HashableValue): value is string => {
  return typeof value === "string";
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
