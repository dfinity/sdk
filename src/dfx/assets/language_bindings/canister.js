import actorInterface from "ic:idl/{project_name}";

export default icHttpAgent.makeActorFactory(actorInterface)({
  canisterId: "{canister_id}",
});
