import actorInterface from "{did_js}";

import {
  generateKeyPair,
  makeActor,
  makeHttpAgent,
} from "{js_user_lib}";

const { publicKey, secretKey } = generateKeyPair();

const httpAgent = makeHttpAgent({
  canisterId: {canister_id},
  senderSecretKey: secretKey,
  senderPubKey: publicKey,
});

const actor = makeActor(actorInterface)(httpAgent);

export default actor;
