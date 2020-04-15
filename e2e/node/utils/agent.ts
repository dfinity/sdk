import {
  HttpAgent,
  generateKeyPair,
  makeAuthTransform,
  makeNonceTransform,
} from '@internet-computer/userlib';

const agent = new HttpAgent({ host: 'http://localhost:8080' });
agent.addTransform(makeNonceTransform());
agent.setAuthTransform(makeAuthTransform(generateKeyPair()));

export default agent;
