import config from "../dfinity.json";
import actorInterface from "../build/canisters/hello/main.js";
import { IDL, makeActor, makeHttpAgent } from "@internet-computer/js-user-library";

(async () => {
  const httpAgent = makeHttpAgent({
    canisterId: config.canisters.hello.canister_id,
  });
  // FIXME: have `makeActor` accept a function?
  const actor = makeActor(actorInterface({ IDL }))(httpAgent);
  const reply = await actor.greet();
  console.log(reply);
})();
