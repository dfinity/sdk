// Canister IDs are represented as an array of bytes in the HTTP handler of the client.
export class CanisterId {
  public static fromText(text: string): CanisterId {
    if (text.startsWith('ic:')) {
      const hex = text.slice(3);
      if (hex.length % 2 === 0 && /^[0-9A-F]+$/.test(hex)) {
        // Remove the checksum from the hexadecimal.
        // TODO: validate the checksum.
        return this.fromHex(hex.slice(0, -2));
      } else {
        throw new Error('Cannot parse canister id: ' + text);
      }
    } else {
      throw new Error('CanisterId not a "ic:" url: ' + text);
    }
  }

  private static fromHex(hex: string): CanisterId {
    return new this(hex);
  }

  protected constructor(private _idHex: string) {}

  public toHex(): string {
    return this._idHex;
  }
  public toText(): string {
    return 'ic:' + this.toHex() + '00';
  }
}
