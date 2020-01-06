import { Buffer } from 'buffer/';
import { lebEncode } from './utils/leb128';

export interface JsonArray extends Array<JsonValue> {}
export interface JsonObject extends Record<string, JsonValue> {}
export type JsonValue = boolean | string | number | JsonArray | JsonObject;

// TODO
// Switch back to Uint8Array once hansl/simple-cbor provides deserialization

// Named `BinaryBlob` as opposed to `Blob` so not to conflict with
// https://developer.mozilla.org/en-US/docs/Web/API/Blob
export type BinaryBlob = Buffer & { __BLOB: never };

export function blobFromHex(hex: string): BinaryBlob {
  return Buffer.from(hex, 'hex') as BinaryBlob;
}

export function blobToHex(blob: BinaryBlob): string {
  return blob.toString('hex');
}

// A Nonce that can be used for calls.
export type Nonce = BinaryBlob & { __nonce__: void };

export function makeNonce(): Nonce {
  return lebEncode(+(+Date.now() + ('' + Math.random()).slice(2, 7))) as Nonce;
}
