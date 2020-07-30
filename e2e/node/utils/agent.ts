import {
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