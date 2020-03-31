export * from './actor';
export { generateKeyPair, makeAuthTransform, makeKeyPair } from './auth';
export * from './canisterId';
export * from './http_agent';
export * from './http_agent_transforms';
export * from './http_agent_types';
export * from './types';

import * as IDL from './idl';
export { IDL };

// TODO The following modules will be a separate library for Candid
import * as UICore from './candid/candid-core';
import * as UI from './candid/candid-ui';
export { UICore, UI };

