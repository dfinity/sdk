export * from './actor';
export * from './agent';
export {
  KeyPair,
  SenderPubKey,
  SenderSecretKey,
  SenderSig,
  generateKeyPair,
  makeAuthTransform,
  makeKeyPair,
} from './auth';
export * from './canisterId';
export * from './http_agent_transforms';
export * from './http_agent_types';
export * from './principal';
export * from './types';

export * from './candid';

import { Agent, HttpAgent } from './agent';
import * as IDL from './idl';
export { IDL };

// TODO The following modules will be a separate library for Candid
import * as UICore from './candid/candid-core';
import * as UI from './candid/candid-ui';
export { UICore, UI };

export interface GlobalInternetComputer {
  ic: {
    agent: Agent;
    HttpAgent: typeof HttpAgent;
    IDL: typeof IDL;
  };
}
