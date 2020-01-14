const changeEndianness = (str: string) => {
        const result = [];
        let len = str.length - 2;
        while (len >= 0) {
          result.push(str.substr(len, 2));
          len -= 2;
        }
        return result.join('');
}

// Canister IDs are represented as u64 in the HTTP handler of the client.
export class CanisterId {
  public static fromText(hex: string): CanisterId {
    if (hex.startsWith('ic:')) {
      // Remove the checksum from the hexadecimal.
      // TODO: validate the checksum.
      return CanisterId.fromText(hex.slice(3, -2));
    }

    console.log("HERE");
    return new this(changeEndianness(hex));
  }

  protected constructor(private _idHex: string) {}

  public toHex(): string {
    return this._idHex;
  }
}
