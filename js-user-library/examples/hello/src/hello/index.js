import config from "../../dfx.json";
import actorInterface from "../../canisters/hello/main.js";

import {
  generateKeyPair,
  makeActor,
  makeHttpAgent,
} from "@internet-computer/js-user-library";

(async () => {
  const { publicKey, secretKey } = generateKeyPair();
  const httpAgent = makeHttpAgent({
    canisterId: config.canisters.hello.deployment_id,
    senderSecretKey: secretKey,
    senderPubKey: publicKey,
  });
  const actor = makeActor(actorInterface)(httpAgent);
  try {
    const reply = await actor.greet();
    console.log(reply);
  } catch (error) {
    console.error(error);
  }
})();
