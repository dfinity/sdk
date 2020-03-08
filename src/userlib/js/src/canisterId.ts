// Canister IDs are represented as an array of bytes in the HTTP handler of the client.
export class CanisterId {
  public static fromText(hex: string): CanisterId {
    if (hex.startsWith('ic:')) {
      // Remove the checksum from the hexadecimal.
      // TODO: validate the checksum.
      return this.fromHex(hex.slice(3, -2));
    } else {
      throw new Error('CanisterId not a "ic:" url: ' + hex);
    }
  }

  private static fromHex(hex: string): CanisterId {
    return new this(hex);
  }

  protected constructor(private _idHex: string) {}

  public toHex(): string {
    return this._idHex;
  }
}
