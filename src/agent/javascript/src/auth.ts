import { Buffer } from 'buffer/';
import * as tweetnacl from 'tweetnacl';
import {
  AuthHttpAgentRequestTransformFn,
  HttpAgentRequest,
  SignedHttpAgentRequest,
} from './http_agent_types';
import { RequestId, requestIdOf } from './request_id';
import { BinaryBlob } from './types';

const domainSeparator = Buffer.from('\x0Aic-request');

export type SenderPubKey = BinaryBlob & { __senderPubKey__: void };
export type SenderSecretKey = BinaryBlob & { __senderSecretKey__: void };
export type SenderSig = BinaryBlob & { __senderSig__: void };

export interface KeyPair {
  publicKey: SenderPubKey;
  secretKey: SenderSecretKey;
}

export function sign(requestId: RequestId, secretKey: SenderSecretKey): SenderSig {
  const bufA = Buffer.concat([domainSeparator, requestId]);
  const signature = tweetnacl.sign.detached(bufA, secretKey);
  return Buffer.from(signature) as SenderSig;
}

export function verify(
  requestId: RequestId,
  senderSig: SenderSig,
  senderPubKey: SenderPubKey,
): boolean {
  const bufA = Buffer.concat([domainSeparator, requestId]);
  return tweetnacl.sign.detached.verify(bufA, senderSig, senderPubKey);
}

export const createKeyPairFromSeed = (seed: Uint8Array): KeyPair => {
  const { publicKey, secretKey } = tweetnacl.sign.keyPair.fromSeed(seed);
  return {
    publicKey: Buffer.from(publicKey),
    secretKey: Buffer.from(secretKey),
  } as KeyPair;
};

// TODO/Note/XXX(eftychis): Unused for the first pass. This provides
// us with key generation for the client.
export function generateKeyPair(): KeyPair {
  const { publicKey, secretKey } = tweetnacl.sign.keyPair();
  return makeKeyPair(publicKey, secretKey);
}

export function makeKeyPair(publicKey: Uint8Array, secretKey: Uint8Array): KeyPair {
  return {
    publicKey: Buffer.from(publicKey),
    secretKey: Buffer.from(secretKey),
  } as KeyPair;
}

export type SigningConstructedFn = (requestId: RequestId, secretKey: SenderSecretKey) => SenderSig;

export function makeAuthTransform(
  keyPair: KeyPair,
  senderSigFn: SigningConstructedFn = sign,
): AuthHttpAgentRequestTransformFn {
  const { publicKey, secretKey } = keyPair;

  const fn = async (r: HttpAgentRequest) => {
    const { body, ...fields } = r;
    const requestId = await requestIdOf(body);
    return {
      ...fields,
      body: {
        content: body,
        sender_pubkey: publicKey,
        sender_sig: senderSigFn(requestId, secretKey),
      },
    } as SignedHttpAgentRequest;
  };

  return fn;
}
