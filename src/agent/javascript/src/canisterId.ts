import { crc8 } from 'crc';
import { blobFromHex } from './types';

function getCrc(hex: string): string {
  return crc8(Buffer.from(hex, 'hex'))
    .toString(16)
    .toUpperCase()
    .padStart(2, '0');
}

// Canister IDs are represented as an array of bytes in the HTTP handler of the client.
export class CanisterId {
  public static fromText(text: string): CanisterId {
    if (text.startsWith('ic:')) {
      text = text.toUpperCase();
      const hex = text.slice(3);
      if (hex.length >= 2 && hex.length % 2 === 0 && /^[0-9A-F]+$/.test(hex)) {
        const id = hex.slice(0, -2);
        const checksum = hex.slice(-2);
        if (checksum !== getCrc(id)) {
          throw new Error('Illegal CanisterId: ' + text);
        }
        return this.fromHex(id);
      } else {
        throw new Error('Cannot parse CanisterId: ' + text);
      }
    } else {
      throw new Error('CanisterId is not a "ic:" url: ' + text);
    }
  }

  public static fromHex(hex: string): CanisterId {
    return new this(hex);
  }

  public readonly _isCanisterId = true;
  protected constructor(private _idHex: string) {}

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
}
