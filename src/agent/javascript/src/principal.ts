import { SenderPubKey } from './auth';
import { BinaryBlob, blobFromHex, blobFromUint8Array, blobToHex } from './types';
import { getCrc } from './utils/getCrc';
import { sha256 } from './utils/sha256';

const SELF_AUTHENTICATING_SUFFIX = 2;

export class Principal {
  public static async selfAuthenticating(publicKey: SenderPubKey): Promise<Principal> {
    const sha = await sha256(publicKey);
    return new this(blobFromUint8Array(new Uint8Array([...sha, 2])));
  }

  public static fromHex(hexNoChecksum: string): Principal {
    return new this(blobFromHex(hexNoChecksum));
  }

  public static fromText(text: string): Principal {
    if (text.startsWith('ic:')) {
      return this.fromHexWithChecksum(text.slice(3));
    } else {
      throw new Error('PrincipalId is not a "ic:" url: ' + text);
    }
  }

  public static fromBlob(blob: BinaryBlob): Principal {
    return new this(blob);
  }

  private static fromHexWithChecksum(hexWithChecksum: string): Principal {
    const hex = hexWithChecksum.toUpperCase();
    if (hex.length >= 2 && hex.length % 2 === 0 && /^[0-9A-F]+$/.test(hex)) {
      const id = hex.slice(0, -2);
      const checksum = hex.slice(-2);
      const crc = getCrc(id);
      if (checksum !== crc) {
        throw new Error(`Invalid checksum for PrincipalId: "ic:${hexWithChecksum}"`);
      }
      return new this(blobFromHex(id));
    } else {
      throw new Error('Cannot parse PrincipalId: ' + hexWithChecksum);
    }
  }

  public readonly _isPrincipal = true;

  protected constructor(private _blob: BinaryBlob) {}

  public toBlob(): BinaryBlob {
    return this._blob;
  }

  public toHash() {
    return this._blob;
  }

  public toHex(): string {
    return blobToHex(this._blob).toUpperCase();
  }

  public toText(): string {
    const token = this.toHex().toUpperCase();
    const crc = getCrc(token);
    return `ic:${token}${crc}`;
  }

  public toString(): string {
    return this.toText();
  }
}
