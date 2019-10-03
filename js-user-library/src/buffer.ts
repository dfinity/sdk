import { Buffer } from "buffer";

export const toHex = (buffer: Buffer): string => {
  return buffer.toString("hex");
};
