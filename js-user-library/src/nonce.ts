import { Buffer } from "buffer";

export const makeNonce = (): Buffer => {
  return makeNonceFromDate(new Date());
};

const makeNonceFromDate = (date: Date): Buffer => {
  const ints = date.getTime().toString().split("").map((x) => parseInt(x, 10));
  return Buffer.from(Uint8Array.from(ints));
};
