import actorInterface from "ic:idl/{project_name}";
import {
  generateKeyPair,
  makeActorFactory,
  makeAuthTransform,
  HttpAgent,
  makeNonceTransform,
} from "ic:userlib";

if (!window.icHttpAgent) {
  const identityIndex = "dfinity-ic-user-identity";
  let k = window.localStorage.getItem(identityIndex);
  let keyPair;
  if (k) {
    keyPair = JSON.parse(k);
  } else {
    keyPair = generateKeyPair();
    // TODO(eftycis): use a parser+an appropriate format to avoid
    // leaking the key when constructing the string for
    // localStorage.
    window.localStorage.setItem(identityIndex, JSON.stringify(keyPair));
  }

  const agent = new HttpAgent({});
  agent.addTransform(makeNonceTransform());
  agent.addTransform(makeAuthTransform(keyPair));

  window.icHttpAgent = agent;
}


const actor = makeActorFactory(actorInterface)({
  canisterId: "{canister_id}",
});

export default actor;
