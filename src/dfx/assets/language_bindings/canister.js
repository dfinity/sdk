import { Actor, HttpAgent } from "@dfinity/agent";

import idlImport from './{canister_name}.did.js';
export const idlFactory = idlImport;
import { canisterId } from './canisterId'

/**
 *
 * @param {string | Principal} canisterId Canister ID of Agent
 * @param {{agentOptions?: import("@dfinity/agent").HttpAgentOptions; actorOptions?: import("@dfinity/agent").ActorConfig}} [options]
 * @return {import("@dfinity/agent").ActorSubclass<import("./{canister_name}.did._SERVICE")>}
 */
 export const createActor = (canisterId, options) => {
  const agent = new HttpAgent({ ...options?.agentOptions });
  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
    ...options.actorOptions,
  });
};
  
/**
 * @type {import("@dfinity/agent").ActorSubclass<import("./{canister_name}.did._SERVICE")>}
 */
 export const {canister_name} = createActor(canisterId);