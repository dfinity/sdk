import * as cbor from 'simple-cbor';
import { SenderPubKey } from './auth';
import { BinaryBlob, blobFromUint8Array } from './types';
import { sha224 } from './utils/sha224';

const SELF_AUTHENTICATING_SUFFIX = 2;

export class Principal {
  public static selfAuthenticating(publicKey: SenderPubKey): Principal {
    const sha = sha224(publicKey);
    return new Principal(blobFromUint8Array(new Uint8Array([...sha, 2])));
  }

  public readonly _isPrincipal = true;

  protected constructor(private _blob: BinaryBlob) {}

  public toBlob(): BinaryBlob {
    return this._blob;
  }
}
