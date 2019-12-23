import actorInterface from "ic:idl/{project_name}";
import {
  generateKeyPair,
  makeActorFactory,
  makeAuthTransform,
  makeHttpAgent,
  makeNonceTransform,
} from "ic:userlib";

if (!window.icHttpAgent) {
  const keyPair = generateKeyPair();
  const agent = makeHttpAgent({});
  agent.addTransform(makeNonceTransform());
  agent.addTransform(makeAuthTransform(keyPair));

  window.icHttpAgent = agent;
}


const actor = makeActorFactory(actorInterface)({
  canisterId: "{canister_id}",
});

export default actor;
