import actorInterface from "ic:idl/{project_name}";
import { generateKeyPair, makeActor, makeHttpAgent } from "ic:userlib";

const { publicKey, secretKey } = generateKeyPair();

const httpAgent = makeHttpAgent({
  canisterId: "{canister_id}",
  senderSecretKey: secretKey,
  senderPubKey: publicKey,
});

const actor = makeActor(actorInterface)(httpAgent);

export default actor;
