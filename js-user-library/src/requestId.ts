import { Buffer } from "buffer/";
import * as buffer from "./buffer";
import { CborValue } from "./cbor";
import { Hex } from "./hex";
import { Request } from "./httpAgent";
import * as int from "./int";
import { Int } from "./int";

export type RequestId = Buffer & { __requestId__: void };

// The spec describes encoding for these types.
// The exception here is integers, which are used in the current implementation
// of the HTTP handler.
type HashableValue = string | Buffer | Int;

export const hash = async (data: Buffer): Promise<Buffer> => {
  const hashed = await crypto.subtle.digest({ name: "SHA-256" }, data.buffer);
  return Buffer.from(hashed);
};

const hashValue = (value: HashableValue): Promise<Buffer> => {
  if (isString(value)) {
    return hashString(value as string);
  } else if (isInt(value)) {
    // HACK: HTTP handler expects canister_id to be an u64 & hashed in this way.
    const hex = int.toHex(value);
    const padded = `${"0000000000000000".slice(hex.length)}${hex}` as Hex;
    return hash(buffer.fromHex(padded));
  } else if (isBuffer) {
    return hash(value as Buffer);
  } else {
    throw new Error(`Attempt to hash a value if unsupported type: ${value}`);
  }
};

const hashString = (value: string): Promise<Buffer> => {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(value);
  return hash(Buffer.from(encoded));
};

const isBuffer = (value: HashableValue): value is Buffer => {
  return Buffer.isBuffer(value);
};

const isInt = (value: HashableValue): value is Int => {
  return typeof value === "number";
};

const isString = (value: HashableValue): value is string => {
  return typeof value === "string";
};

const concat = (bs: Array<Buffer>): Buffer => {
  const folded = bs.reduce((state: Uint8Array, b: Buffer): Uint8Array => {
    return new Uint8Array([
      ...state,
      ...(new Uint8Array(b.buffer)),
    ]);
  }, new Uint8Array());
  return Buffer.from(folded);
};

export const requestIdOf = async (request: Request): Promise<RequestId> => {
  const { sender_pubkey, sender_sig, ...rest } = request;

  const hashed: Array<Promise<[Buffer, Buffer]>> = Object
    .entries(rest)
    .map(async ([key, value]: [string, CborValue]) => {
      const hashedKey = await hashString(key);
      const hashedValue = await hashValue(value as HashableValue);

      return [
        hashedKey,
        hashedValue,
      ] as [Buffer, Buffer];
    });

  const traversed: Array<[Buffer, Buffer]> = await Promise.all(hashed);

  const sorted: Array<[Buffer, Buffer]> = traversed
    .sort(([k1, v1], [k2, v2]) => {
      return Buffer.compare(k1, k2);
    });

  const concatenated: Buffer = concat(sorted.map(concat));
  return hash(concatenated) as Promise<RequestId>;
};
