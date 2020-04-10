import { Buffer } from 'buffer/';
import { sign as naclSign } from 'tweetnacl';
import { AuthHttpAgentRequestTransformFn, HttpAgentRequest, SignedHttpAgentRequest} from './http_agent_types';
import { RequestId, requestIdOf } from './request_id';
import { BinaryBlob } from './types';

export type SenderPubKey = BinaryBlob & { __senderPubKey__: void };
export type SenderSecretKey = BinaryBlob & { __senderSecretKey__: void };
export type SenderSig = BinaryBlob & { __senderSig__: void };

export interface KeyPair {
  publicKey: SenderPubKey;
  secretKey: SenderSecretKey;
}

export const sign = (secretKey: SenderSecretKey) => (requestId: RequestId): SenderSig => {
  const signature = naclSign.detached(requestId, secretKey);
  return Buffer.from(signature) as SenderSig;
};

export function verify(
  requestId: RequestId,
  senderSig: SenderSig,
  senderPubKey: SenderPubKey,
): boolean {
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
export function generateKeyPair(): KeyPair {
  const { publicKey, secretKey } = naclSign.keyPair();
  return makeKeyPair(publicKey, secretKey);
}

export function makeKeyPair(publicKey: Uint8Array, secretKey: Uint8Array): KeyPair {
  return {
    publicKey: Buffer.from(publicKey),
    secretKey: Buffer.from(secretKey),
  } as KeyPair;
}

export type SigningConstructedFn = (
  secretKey: SenderSecretKey,
) => (requestId: RequestId) => SenderSig;

export function makeAuthTransform(
  keyPair: KeyPair,
  senderSigFn: SigningConstructedFn = sign,
): AuthHttpAgentRequestTransformFn {
  const { publicKey, secretKey } = keyPair;
  const signFn = senderSigFn(secretKey);

  const fn = async (r: HttpAgentRequest) => {
    const {body, ...fields } = r;
    const requestId = await requestIdOf(body);
    return { ...fields,
    body: {
        content: body,
        sender_pubkey: publicKey,
        sender_sig: signFn(requestId),
    } } as SignedHttpAgentRequest;
  };

  return fn;
}
