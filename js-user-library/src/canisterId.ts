import BigNumber from 'bignumber.js';

// Canister IDs are represented as u64 in the HTTP handler of the client.
export class CanisterId {
  public static fromHex(hex: string) {
    return new this(new BigNumber(hex, 16));
  }

  constructor(private _id: BigNumber) {}

  public toHex(): string {
    return this._id.toString(16);
  }
}
