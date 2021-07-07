import { Actor, HttpAgent } from "@dfinity/agent";

import idlImport from './{canister_name}.did.js';
export const idlFactory = idlImport;
export const canisterId = process.env.{canister_name_uppercase}_CANISTER_ID;

/**
 *
 * @param {string | Principal} canisterId Canister ID of Agent
 * @param {{agentOptions?: import("@dfinity/agent").HttpAgentOptions; actorOptions?: import("@dfinity/agent").ActorConfig}} [options]
 * @return {import("@dfinity/agent").ActorSubclass<import("./{canister_name}.did")._SERVICE>}
 */
 export const createActor = (canisterId, options) => {
  const agent = new HttpAgent({ ...options?.agentOptions });
  
  // Fetch root key for certificate validation during development
  if(process.env.NODE_ENV !== "production") agent.fetchRootKey();

  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
    ...options?.actorOptions,
  });
};
  
/**
 * @type {import("@dfinity/agent").ActorSubclass<import("./{canister_name}.did")._SERVICE>}
 */
 export const {canister_name} = createActor(canisterId);