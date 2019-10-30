import { Buffer } from "buffer/";
import { Hex } from "./hex";

// Named `BinaryBlob` as opposed to `Blob` so not to conflict with
// https://developer.mozilla.org/en-US/docs/Web/API/Blob
export type BinaryBlob = Uint8Array & { __blob__: void };

export const fromHex = (hex: Hex): BinaryBlob => {
  return new Uint8Array(Buffer.from(hex, "hex").buffer) as BinaryBlob;
};

export const toHex = (blob: BinaryBlob): Hex => {
  return Buffer.from(blob).toString("hex") as Hex;
};
