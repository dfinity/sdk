// Canister IDs are represented as u64 in the HTTP handler of the client.
export class CanisterId {
  public static fromHex(hex: string): CanisterId {
    if (hex.startsWith('ic:')) {
      return CanisterId.fromHex(hex.slice(3, -2));
    }

    return new this(hex.padStart(16, '0'));
  }

  protected constructor(private _idHex: string) {}

  public toHex(): string {
    return this._idHex;
  }
}
