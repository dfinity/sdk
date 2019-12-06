import { Buffer } from 'buffer/';

// TODO
// Switch back to Uint8Array once hansl/simple-cbor provides deserialization

// Named `BinaryBlob` as opposed to `Blob` so not to conflict with
// https://developer.mozilla.org/en-US/docs/Web/API/Blob
export type BinaryBlob = Buffer;

export const fromHex = (hex: string): BinaryBlob => {
  return Buffer.from(hex, 'hex') as BinaryBlob;
};

export const toHex = (blob: BinaryBlob) => {
  return blob.toString('hex');
};
