import { Buffer } from "buffer";

export const toHex = (buffer: Buffer): string => {
  return Array
    .from(buffer)
    .map((x) => `00${x.toString(16)}`.slice(-2))
    .join("");
};
