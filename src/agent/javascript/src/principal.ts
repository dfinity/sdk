import * as cbor from 'simple-cbor';
import { SenderPubKey } from './auth';
import { BinaryBlob, blobFromUint8Array } from './types';
import { sha256 } from './utils/sha256';

const SELF_AUTHENTICATING_SUFFIX = 2;

export class Principal {
  public static async selfAuthenticating(publicKey: SenderPubKey): Promise<Principal> {
    const sha = await sha256(publicKey);
    return new Principal(blobFromUint8Array(new Uint8Array([...sha, 2])));
  }

  public readonly _isPrincipal = true;

  protected constructor(private _blob: BinaryBlob) {}

  public toBlob(): BinaryBlob {
    return this._blob;
  }
}
