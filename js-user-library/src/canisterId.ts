import { Hex } from "./hex";

// Canister IDs are represented as u64. We use `borc` for CBOR encoding and
// decoding, which supports `bignumber.js` out of the box.
export class CanisterId {
  public hex: Hex;

  constructor(hex: Hex) {
    this.hex = hex;
  }

  public encodeCBOR(gen: { pushAny: (_: any) => boolean }) {
    // TODO: encode
    return gen.pushAny(this.hex);
  }
}
