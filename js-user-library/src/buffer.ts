import { Buffer } from "buffer/";
import { Hex } from "./hex";

export const fromHex = (hex: Hex): Buffer => {
  return Buffer.from(hex, "hex");
};

export const toHex = (buffer: Buffer): string => {
  return buffer.toString("hex");
};
