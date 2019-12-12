import { Buffer } from 'buffer/';
import { sign as naclSign } from 'tweetnacl';
import { RequestId } from './request_id';
import { SenderPubKey } from './sender_pub_key';
import { SenderSecretKey } from './sender_secret_key';
import { SenderSig } from './sender_sig';

export interface KeyPair {
  publicKey: SenderPubKey;
  secretKey: SenderSecretKey;
}

export const sign = (secretKey: SenderSecretKey) => (requestId: RequestId): SenderSig => {
  const signature = naclSign.detached(requestId, secretKey);
  return Buffer.from(signature) as SenderSig;
};

export function verify(requestId: RequestId, senderSig: SenderSig, senderPubKey: SenderPubKey) {
  return naclSign.detached.verify(requestId, senderSig, senderPubKey);
}

export const createKeyPairFromSeed = (seed: Uint8Array): KeyPair => {
  const { publicKey, secretKey } = naclSign.keyPair.fromSeed(seed);
  return {
    publicKey: Buffer.from(publicKey),
    secretKey: Buffer.from(secretKey),
  } as KeyPair;
};

// TODO/Note/XXX(eftychis): Unused for the first pass. This provides
// us with key generation for the client.
export const generateKeyPair = (): KeyPair => {
  const { publicKey, secretKey } = naclSign.keyPair();
  return {
    publicKey: Buffer.from(publicKey),
    secretKey: Buffer.from(secretKey),
  } as KeyPair;
};
