import borc from 'borc';
import { Buffer } from 'buffer/';
import { CanisterId } from './canisterId';
import { BinaryBlob, blobFromHex, blobToHex } from './types';

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

const changeEndianness = (str: string): string => {
  const result = [];
  let len = str.length - 2;
  while (len >= 0) {
    result.push(str.substr(len, 2));
    len -= 2;
  }
  return result.join('');
};

async function hashValue(value: unknown): Promise<Buffer> {
  if (value instanceof borc.Tagged) {
    return hashValue(value.value);
  } else if (typeof value === 'string') {
    return hashString(value);
  } else if (value instanceof CanisterId) {
    // HTTP handler expects canister_id to be an u64 & hashed in this way.
    // work-around for endianness problem until we switch to blobs
    return hash(blobFromHex(changeEndianness(value.toHex())));
  } else if (value instanceof Buffer) {
    return hash(new Uint8Array(value) as BinaryBlob);
  } else if (value instanceof Uint8Array || value instanceof ArrayBuffer) {
    return hash(new Uint8Array(value) as BinaryBlob);
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
  // While the type signature of this function ensures the fields we care about
  // are present, it does not prevent additional fields from being provided,
  // including the fields used for authentication that we must omit when
  // calculating the request ID. This is by design, since requests are expected
  // to have more than just the common fields. As a result, we need to explictly
  // ignore the authentication fields.
  const { sender_pubkey, sender_sig, ...fields } = request;

  const hashed: Array<Promise<[BinaryBlob, BinaryBlob]>> = Object.entries(fields).map(
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
