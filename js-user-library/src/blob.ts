import { Buffer } from "buffer/";
import { Hex } from "./hex";

export const fromHex = (hex: Hex): Uint8Array => {
  return new Uint8Array(Buffer.from(hex, "hex").buffer);
};

export const toHex = (blob: Uint8Array): string => {
  return Buffer.from(blob).toString("hex");
};
