import { Buffer } from "buffer/";
import { CborValue } from "./cbor";
import { Request, RequestId } from "./httpAgent";

// The spec describes encoding for these types.
type HashableValue = string | Buffer;

export const hash = async (data: Buffer): Promise<Buffer> => {
  const hashed = await crypto.subtle.digest({ name: "SHA-256" }, data.buffer);
  return Buffer.from(hashed);
};

const hashValue = (value: HashableValue): Promise<Buffer> => {
  return isString(value)
    ? hashString(value as string)
    : hash(value as Buffer);
};

const hashString = (value: string): Promise<Buffer> => {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(value);
  return hash(Buffer.from(encoded));
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
      // Behavior is undefined for ints and records. The spec only describes
      // encoding for strings and binary blobs.
      const hashedValue = await hashValue(value as string | Buffer);

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
  return hash(concatenated);
};
