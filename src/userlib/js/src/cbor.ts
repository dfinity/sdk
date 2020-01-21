// tslint:disable:max-classes-per-file
// This file is based on:
// tslint:disable-next-line: max-line-length
// https://github.com/dfinity-lab/dfinity/blob/9bca65f8edd65701ea6bdb00e0752f9186bbc893/docs/spec/public/index.adoc#cbor-encoding-of-requests-and-responses
import borc from 'borc';
import { Buffer } from 'buffer/';
import * as cbor from 'simple-cbor';
import { CborEncoder, SelfDescribeCborSerializer } from 'simple-cbor';
import { CanisterId } from './canisterId';
import { BinaryBlob } from './types';

// We are using hansl/simple-cbor for CBOR serialization, to avoid issues with
// encoding the uint64 values that the HTTP handler of the client expects for
// canister IDs. However, simple-cbor does not yet provide deserialization so
// we are using `BigNumber` and `Buffer` types instead of `BigInt` and
// `Uint8Array` (respectively) so that we can use the dignifiedquire/borc CBOR
// decoder.

class CanisterIdEncoder implements CborEncoder<CanisterId> {
  public get name() {
    return 'CanisterId';
  }

  public get priority() {
    return 0;
  }

  public match(value: any): boolean {
    return value instanceof CanisterId;
  }

  public encode(v: CanisterId): cbor.CborValue {
    return cbor.value.u64(this.changeEndianness(v.toHex()), 16);
  }

  // Drop once we use https://github.com/dfinity-lab/dfinity/pull/2286
  private changeEndianness(str: string): string {
    const result = [];
    let len = str.length - 2;
    while (len >= 0) {
      result.push(str.substr(len, 2));
      len -= 2;
    }
    return result.join('');
  }
}

class BufferEncoder implements CborEncoder<Buffer> {
  public get name() {
    return 'Buffer';
  }

  public get priority() {
    return 1;
  }

  public match(value: any): boolean {
    return value instanceof Buffer;
  }

  public encode(v: Buffer): cbor.CborValue {
    return cbor.value.bytes(new Uint8Array(v.buffer));
  }
}

const serializer = SelfDescribeCborSerializer.withDefaultEncoders();
serializer.addEncoder(new CanisterIdEncoder());
serializer.addEncoder(new BufferEncoder());

export enum CborTag {
  Uint64LittleEndian = 71,
  Semantic = 55799,
}

export const encode = (value: any): BinaryBlob => {
  return Buffer.from(serializer.serialize(value)) as BinaryBlob;
};

export function decode<T>(input: Uint8Array): T {
  const decoder = new borc.Decoder({
    size: input.byteLength,
    tags: {
      [CborTag.Semantic]: (value: T): T => value,
    },
  });
  const result = decoder.decodeFirst(input);
  if (result.hasOwnProperty('canister_id')) {
    result.canister_id = CanisterId.fromText(result.canister_id.toString(16));
  }
  return result;
}
