import { Buffer } from 'buffer/';
import { Hex } from './hex';

// TODO
// Switch back to Uint8Array once hansl/simple-cbor provides deserialization

// Named `BinaryBlob` as opposed to `Blob` so not to conflict with
// https://developer.mozilla.org/en-US/docs/Web/API/Blob
export type BinaryBlob = Buffer & { __blob__: void };

export const fromHex = (hex: Hex): BinaryBlob => {
  return Buffer.from(hex, 'hex') as BinaryBlob;
};

export const toHex = (blob: BinaryBlob): Hex => {
  return blob.toString('hex') as Hex;
};
