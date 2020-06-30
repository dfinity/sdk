import actorInterface from "ic:idl/{project_name}";

export default ic.agent.makeActorFactory(actorInterface)({
  canisterId: "{canister_id}",
});
