import tweetnacl from "tweetnacl";
import { RequestId } from "./requestId";
import { SenderPubKey } from "./senderPubKey";
import { SenderSecretKey } from "./senderSecretKey";
import { SenderSig } from "./senderSig";


export interface KeyPair {
  publicKey: SenderPubKey;
  secretKey: SenderSecretKey;
}

export const sign = (
  secretKey: SenderSecretKey,
) => (
  requestId: RequestId,
): SenderSig => {
  return tweetnacl.sign.detached(requestId, secretKey) as SenderSig;
};

export const verify = (
  requestId: RequestId,
  senderSig: SenderSig,
  senderPubKey: SenderPubKey,
): boolean => {
  return tweetnacl.sign.detached.verify(requestId, senderSig, senderPubKey);
};

export const createKeyPairFromSeed = (
  seed: Uint8Array,
): KeyPair => {
 return tweetnacl.sign.keyPair.fromSeed(seed) as KeyPair;

};

// TODO/Note/XXX(eftychis): Unused for the first pass. This provides
// us with key generation for the client.
export const generateKeyPair = (): KeyPair =>
  tweetnacl.sign.keyPair() as KeyPair;
