// We use `borc` to encode CBOR, and it has built-in support for `bignumber.js`
import BigNumber from "bignumber.js";
import { Int } from "./int";

export const toHex = (bytes: Array<Int>): string => {
  return bytes.map((x) => `00${x.toString(16)}`.slice(-2)).join("");
};

export const toBigInt = (bytes: Array<Int>): BigNumber => {
  return (new BigNumber(`0x${toHex(bytes)}`));
};
