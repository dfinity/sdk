import borc from 'borc';
import { Buffer } from 'buffer/';
import { CanisterId } from './canisterId';
import { BinaryBlob, blobFromHex, blobToHex } from './types';
import { lebEncode } from './utils/leb128';

export type RequestId = BinaryBlob & { __requestId__: void };
export function toHex(requestId: RequestId): string {
  return blobToHex(requestId);
}

export async function hash(data: BinaryBlob): Promise<BinaryBlob> {
  const hashed: ArrayBuffer = await crypto.subtle.digest(
    {
      name: 'SHA-256',
    },
    data.buffer,
  );
  return Buffer.from(hashed) as BinaryBlob;
}

async function hashValue(value: unknown): Promise<Buffer> {
  if (value instanceof borc.Tagged) {
    return hashValue(value.value);
  } else if (typeof value === 'string') {
    return hashString(value);
  } else if (typeof value === 'number') {
    return hash(lebEncode(value) as BinaryBlob);
  } else if (Buffer.isBuffer(value)) {
    return hash(new Uint8Array(value) as BinaryBlob);
  } else if (value instanceof Uint8Array || value instanceof ArrayBuffer) {
    return hash(new Uint8Array(value) as BinaryBlob);
  } else if (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as any).toHash === 'function'
  ) {
    return Promise.resolve((value as any).toHash()).then((x) => hashValue(x));
  } else if (value instanceof Promise) {
    return value.then((x) => hashValue(x));
  } else {
    throw new Error(`Attempt to hash a value of unsupported type: ${value}`);
  }
}

const hashString = (value: string): Promise<BinaryBlob> => {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(value);
  return hash(Buffer.from(encoded) as BinaryBlob);
};

const concat = (bs: BinaryBlob[]): BinaryBlob => {
  return bs.reduce((state: Uint8Array, b: BinaryBlob): Uint8Array => {
    return new Uint8Array([...state, ...b]);
  }, new Uint8Array()) as BinaryBlob;
};

export const requestIdOf = async (request: Record<string, any>): Promise<RequestId> => {
  const hashed: Array<Promise<[BinaryBlob, BinaryBlob]>> = Object.entries(request).map(
    async ([key, value]: [string, unknown]) => {
      const hashedKey = await hashString(key);
      const hashedValue = await hashValue(value);

      return [hashedKey, hashedValue] as [BinaryBlob, BinaryBlob];
    },
  );

  const traversed: Array<[BinaryBlob, BinaryBlob]> = await Promise.all(hashed);

  const sorted: Array<[BinaryBlob, BinaryBlob]> = traversed.sort(([k1, v1], [k2, v2]) => {
    return Buffer.compare(Buffer.from(k1), Buffer.from(k2));
  });

  const concatenated: BinaryBlob = concat(sorted.map(concat));
  const requestId = (await hash(concatenated)) as RequestId;
  return requestId;
};
