import { BinaryBlob } from "./blob";

export type Nonce = BinaryBlob & { __nonce__: void };

export const makeNonce = (): Nonce => {
  return makeNonceFromDate(new Date());
};

const makeNonceFromDate = (date: Date): Nonce => {
  const ints = date.getTime().toString().split("").map((x) => parseInt(x, 10));
  return Uint8Array.from(ints) as Nonce;
};
