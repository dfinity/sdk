import { Buffer } from "buffer";
import { CborValue } from "./cbor";
import { Request, RequestId } from "./httpAgent";
import { Int } from "./int";

// The spec describes encoding for these types.
type HashableValue = string | Array<Int>;

export const hash = async (data: ArrayBuffer): Promise<ArrayBuffer> => {
  return crypto.subtle.digest({ name: "SHA-256" }, data);
};

const hashValue = (value: HashableValue): Promise<ArrayBuffer> => {
  return isString(value)
    ? hashString(value as string)
    : hashBlob(value as Array<Int>);
};

const hashBlob = (value: Array<Int>): Promise<ArrayBuffer> => {
  return hash(new Uint8Array(value));
};

const hashString = (value: string): Promise<ArrayBuffer> => {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(value);
  return hash(encoded);
};

const isString = (value: HashableValue): value is string => {
  return typeof value === "string";
};

const concat = (bs: Array<ArrayBuffer>): ArrayBuffer => {
  const folded = bs.reduce((state: Uint8Array, b: ArrayBuffer): Uint8Array => {
    return new Uint8Array([
      ...state,
      ...(new Uint8Array(b)),
    ]);
  }, new Uint8Array());
  return folded.buffer;
};

export const requestIdOf = async (request: Request): Promise<RequestId> => {
  const { sender_pubkey, sender_sig, ...rest } = request;

  const hashed: Array<Promise<[ArrayBuffer, ArrayBuffer]>> = Object
    .entries(rest)
    .map(async ([key, value]: [string, CborValue]) => {
      const hashedKey = await hashString(key);
      // Behavior is undefined for ints and records. The spec only describes
      // encoding for strings and binary blobs.
      const hashedValue = await hashValue(value as string | Array<Int>);

      return [
        hashedKey,
        hashedValue,
      ] as [ArrayBuffer, ArrayBuffer];
    });

  const traversed: Array<[ArrayBuffer, ArrayBuffer]> = await Promise.all(
    hashed,
  );

  const sorted: Array<[ArrayBuffer, ArrayBuffer]> = traversed
    .sort(([k1, v1], [k2, v2]) => {
      return Buffer.compare(Buffer.from(k1), Buffer.from(k2));
    });

  const concatenated: ArrayBuffer = concat(sorted.map(concat));
  const buffer = await hash(concatenated);
  return Array.from(new Uint8Array(buffer)) as Array<Int>;
};
