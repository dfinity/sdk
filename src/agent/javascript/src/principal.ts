import { SenderPubKey } from './auth';
import { BinaryBlob, blobFromHex, blobFromUint8Array, blobToHex } from './types';
import { getCrc32 } from './utils/getCrc';
import { sha224 } from './utils/sha224';
var base32 = require('base32.js');

const SELF_AUTHENTICATING_SUFFIX = 2;

export class Principal {
  public static selfAuthenticating(publicKey: SenderPubKey): Principal {
    const sha = sha224(publicKey);
    return new this(blobFromUint8Array(new Uint8Array([...sha, 2])));
  }

  public static fromHex(hexNoChecksum: string): Principal {
    // return this.fromHexMaybeChecksum(hexNoChecksum, false);
    return new this(blobFromHex(hexNoChecksum));
  }

  public static fromText(text: string): Principal {
    return this.fromHexMaybeChecksum(text, true);
  }

  public static fromBlob(blob: BinaryBlob): Principal {
    return new this(blob);
  }

  private static fromHexMaybeChecksum(hexWithChecksum: string, hasChecksum: boolean): Principal {
    let canisterIdNoDash = hexWithChecksum.toLowerCase().replace(/-/g, '');

    let decoder = new base32.Decoder({ type: 'rfc4648', lc: false });
    let result = decoder.write(canisterIdNoDash).finalize();
    let arr = new Uint8Array(result);

    if (hasChecksum) {
      arr = arr.slice(4, arr.length);
    }
    let blob = blobFromUint8Array(arr);
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
    return blobToHex(this._blob).toUpperCase();
  }

  public toText(): string {
    let checksumArrayBuf = new ArrayBuffer(4);
    let view = new DataView(checksumArrayBuf);
    view.setUint32(0, getCrc32(this.toHex().toLowerCase()), false);
    const checksum = Uint8Array.from(Buffer.from(checksumArrayBuf));

    let bytes = Uint8Array.from(this._blob);
    let array = new Uint8Array([...checksum, ...bytes]);

    let encoder = new base32.Encoder({ type: 'rfc4648', lc: false });
    let result = encoder.write(array).finalize().toLowerCase();
    return result.match(/.{1,5}/g).join('-');
  }

  public toString(): string {
    return this.toText();
  }
}
