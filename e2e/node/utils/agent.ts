import {
  CanisterId,
  HttpAgent,
  Principal,
  generateKeyPair,
  makeAuthTransform,
  makeNonceTransform,
} from '@dfinity/agent';

const keyPair = generateKeyPair();
const principal = Principal.selfAuthenticating(keyPair.publicKey);

export const httpAgent = new HttpAgent({ host: 'http://localhost:8080', principal });
httpAgent.addTransform(makeNonceTransform());
httpAgent.setAuthTransform(makeAuthTransform(keyPair));

export function canisterIdFactory() {
  const counterCanisterIdHex = (+new Date() % 0xFFFFFF).toString(16)
    + (Math.floor(Math.random() * 256)).toString(16);
  return CanisterId.fromHex(counterCanisterIdHex);
}
