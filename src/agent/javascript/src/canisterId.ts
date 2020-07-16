import { crc8 } from 'crc';
import { BinaryBlob, blobFromHex, blobToHex } from './types';

function getCrc(hex: string): string {
  return crc8(Buffer.from(hex, 'hex')).toString(16).toUpperCase().padStart(2, '0');
}

// Canister IDs are represented as an array of bytes in the HTTP handler of the client.
export class CanisterId {
  public static fromText(text: string): CanisterId {
    if (text.startsWith('ic:')) {
      return this.fromHexWithChecksum(text.slice(3));
    } else {
      throw new Error('CanisterId is not a "ic:" url: ' + text);
    }
  }

  public static fromHexWithChecksum(hexWithChecksum: string): CanisterId {
    const hex = hexWithChecksum.toUpperCase();
    if (hex.length >= 2 && hex.length % 2 === 0 && /^[0-9A-F]+$/.test(hex)) {
      const id = hex.slice(0, -2);
      const checksum = hex.slice(-2);
      if (checksum !== getCrc(id)) {
        throw new Error(`Invalid checksum for CanisterId: "ic:${hexWithChecksum}"`);
      }
      return new this(id);
    } else {
      throw new Error('Cannot parse CanisterId: ' + hexWithChecksum);
    }
  }

  public static fromHex(hex: string): CanisterId {
    return new this(hex.toUpperCase());
  }

  public static fromBlob(blob: BinaryBlob): CanisterId {
    return new this(blobToHex(blob));
  }

  public readonly _isCanisterId = true;
  protected constructor(private _idHex: string) {}

  public toBlob() {
    return blobFromHex(this._idHex);
  }
  public toHash() {
    return blobFromHex(this._idHex);
  }
  public toHex(): string {
    return this._idHex;
  }
  public toText(): string {
    const crc = getCrc(this._idHex);
    return 'ic:' + this.toHex() + crc;
  }
  public toString(): string {
    return this.toText();
  }
}
