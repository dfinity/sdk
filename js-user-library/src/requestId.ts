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
  const encoded = encoder.encode(value === "canister_id" ? "callee" : value);
  return hash(encoded);
};

const isString = (value: HashableValue): value is string => {
  return typeof value === "string";
};

export const requestIdOf = async (request: Request): Promise<RequestId> => {
  const { sender_pubkey, sender_sig, ...rest } = request;

  const hashed = Object
    .entries(rest)
    .map(async ([key, value]: [string, CborValue]) => {
      const hashedKey = await hashString(key);
      // Behavior is undefined for ints and records. The spec only describes
      // encoding for strings and binary blobs.
      const hashedValue = await hashValue(value as string | Array<Int>);

      return [
        Buffer.from(hashedKey),
        Buffer.from(hashedValue),
      ] as [Buffer, Buffer];
    });

  const traversed = await Promise.all(hashed);
  const sorted = traversed.sort(([k1, v1], [k2, v2]) => Buffer.compare(k1, k2));
  const concatenated = Buffer.concat(sorted.map(Buffer.concat));
  const buffer = await hash(concatenated.buffer);
  return Array.from(new Uint8Array(buffer)) as Array<Int>;
};
