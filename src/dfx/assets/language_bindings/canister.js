import { Actor, HttpAgent, makeNonce, makeNonceTransform } from "@dfinity/agent";

// Imports and re-exports candid interface
import { idlFactory } from './{canister_name}.did.js';
export { idlFactory } from './{canister_name}.did.js';
// CANISTER_ID is replaced by webpack based on node environment
export const canisterId = process.env.{canister_name_uppercase}_CANISTER_ID;

/**
 * 
 * @param {string | import("@dfinity/principal").Principal} canisterId Canister ID of Agent
 * @param {{agentOptions?: import("@dfinity/agent").HttpAgentOptions; actorOptions?: import("@dfinity/agent").ActorConfig; useNonceForUpdates?:boolean; nonceFn?:()=> import("@dfinity/agent").Nonce}} [options]
 * @return {import("@dfinity/agent").ActorSubclass<import("./{canister_name}.did.js")._SERVICE>}
 */
 export const createActor = (canisterId, options) => {
  const {useNonceForUpdates = true, nonceFn} = options ?? {};
  const agent = new HttpAgent({ ...options?.agentOptions });
  if(useNonceForUpdates) {
    // By default we will set a unique nonce so that update calls with
    // the same parameters will be made unique through that unique nonce.
    agent.addTransform(makeNonceTransform(nonceFn??makeNonce));
  }

  // Fetch root key for certificate validation during development
  if(process.env.NODE_ENV !== "production") {
    agent.fetchRootKey().catch(err=>{
      console.warn("Unable to fetch root key. Check to ensure that your local replica is running");
      console.error(err);
    });
  }

  // Creates an actor with using the candid interface and the HttpAgent
  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
    ...options?.actorOptions,
  });
};
  
/**
 * A ready-to-use agent for the {canister_name} canister
 * @type {import("@dfinity/agent").ActorSubclass<import("./{canister_name}.did.js")._SERVICE>}
 */
 export const {canister_name} = createActor(canisterId);
