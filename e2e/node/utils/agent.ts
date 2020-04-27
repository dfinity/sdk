import {
  HttpAgent,
  generateKeyPair,
  makeAuthTransform,
  makeNonceTransform,
} from '@dfinity/agent';

const agent = new HttpAgent({ host: 'http://localhost:8080' });
agent.addTransform(makeNonceTransform());
agent.setAuthTransform(makeAuthTransform(generateKeyPair()));

export default agent;
