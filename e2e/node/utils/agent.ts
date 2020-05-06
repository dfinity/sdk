import {
  HttpAgent,
  Principal,
  generateKeyPair,
  makeAuthTransform,
  makeNonceTransform,
} from '@dfinity/agent';

const keyPair = generateKeyPair();
const principal = Principal.selfAuthenticating(keyPair.publicKey);

const agent = new HttpAgent({ host: 'http://localhost:8080', principal });
agent.addTransform(makeNonceTransform());
agent.setAuthTransform(makeAuthTransform(keyPair));

export default agent;
