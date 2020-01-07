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
  var k = window.localStorage.getItem(identityIndex);
  var k = JSON.parse(k);
  if (!k){
    const keyPair= generateKeyPair();
    var jsonValue = JSON.stringify(keyPair);
    window.localStorage.setItem("dfinity-ic-user-identity", jsonValue);
  } else {
    const keyPair = k;
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
