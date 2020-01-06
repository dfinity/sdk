// Canister IDs are represented as u64 in the HTTP handler of the client.
export class CanisterId {
  public static fromHex(hex: string) {
    return new this(hex);
  }

  constructor(private _idHex: string) {}

  public toHex(): string {
    return this._idHex;
  }
}
