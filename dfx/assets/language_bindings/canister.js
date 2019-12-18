import actorInterface from "ic:idl/{project_name}";
import { makeActorFactory, makeHttpAgent } from "ic:userlib";

window.icHttpAgent = window.icHttpAgent || makeHttpAgent({});

const actor = makeActorFactory(actorInterface)({
  canisterId: "{canister_id}",
});

export default actor;
