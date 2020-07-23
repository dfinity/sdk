import { SenderPubKey } from './auth';
import { getCrc } from './getCrc';
import { BinaryBlob, blobFromHex, blobFromUint8Array, blobToHex } from './types';
import { sha256 } from './utils/sha256';

const SELF_AUTHENTICATING_SUFFIX = 2;

export class Principal {
  public static async selfAuthenticating(publicKey: SenderPubKey): Promise<Principal> {
    const sha = await sha256(publicKey);
    return new Principal(blobFromUint8Array(new Uint8Array([...sha, 2])));
  }

  public static fromHex(hexNoChecksum: string): Principal {
    return new this(blobFromHex(hexNoChecksum));
  }

  public static fromHexWithChecksum(hexWithChecksum: string): Principal {
    const hex = hexWithChecksum.toUpperCase();
    if (hex.length >= 2 && hex.length % 2 === 0 && /^[0-9A-F]+$/.test(hex)) {
      const id = hex.slice(0, -2);
      const checksum = hex.slice(-2);
      const crc = getCrc(id);
      if (checksum !== crc) {
        throw new Error(`Invalid checksum for PrincipalId: "ic:${hexWithChecksum}"`);
      }
      // NB: need to verify this
      return new this(blobFromHex(id));
    } else {
      throw new Error('Cannot parse PrincipalId: ' + hexWithChecksum);
    }
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

  public readonly _isPrincipal = true;

  protected constructor(private _blob: BinaryBlob) {}

  public toBlob(): BinaryBlob {
    return this._blob;
  }

  public toHash() {
    return this._blob;
  }

  public toHex(): string {
    return blobToHex(this._blob);
  }

  public toText(): string {
    const crc = getCrc(this.toHex());
    return 'ic:' + this.toHex() + crc;
  }

  public toString(): string {
    return this.toText();
  }
}
