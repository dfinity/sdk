import base32 from 'base32.js';
import { SenderPubKey } from './auth';
import { BinaryBlob, blobFromHex, blobFromUint8Array, blobToHex } from './types';
import { getCrc32 } from './utils/getCrc';
import { sha224 } from './utils/sha224';

const SELF_AUTHENTICATING_SUFFIX = 2;

export class Principal {
  public static selfAuthenticating(publicKey: SenderPubKey): Principal {
    const sha = sha224(publicKey);
    return new this(blobFromUint8Array(new Uint8Array([...sha, 2])));
  }

  public static fromHex(hex: string): Principal {
    return new this(blobFromHex(hex));
  }

  public static fromText(text: string): Principal {
    const canisterIdNoDash = text.toLowerCase().replace(/-/g, '');

    const decoder = new base32.Decoder({ type: 'rfc4648', lc: false });
    const result = decoder.write(canisterIdNoDash).finalize();
    let arr = new Uint8Array(result);
    arr = arr.slice(4, arr.length);

    return new this(blobFromUint8Array(arr));
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
    return blobToHex(this._blob).toUpperCase();
  }

  public toText(): string {
    const checksumArrayBuf = new ArrayBuffer(4);
    const view = new DataView(checksumArrayBuf);
    view.setUint32(0, getCrc32(this.toHex().toLowerCase()), false);
    const checksum = Uint8Array.from(Buffer.from(checksumArrayBuf));

    const bytes = Uint8Array.from(this._blob);
    const array = new Uint8Array([...checksum, ...bytes]);

    const encoder = new base32.Encoder({ type: 'rfc4648', lc: false });
    const result = encoder.write(array).finalize().toLowerCase();
    const matches = result.match(/.{1,5}/g);
    return matches ? matches.join('-') : '';
  }
}
