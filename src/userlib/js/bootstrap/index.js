import {
  HttpAgent,
  generateKeyPair,
  makeActorFactory,
  makeAuthTransform,
  makeKeyPair,
  makeNonceTransform,
} from "../out";

const identityIndex = "dfinity-ic-user-identity";
let k = window.localStorage.getItem(identityIndex);
let keyPair;
if (k) {
  keyPair = JSON.parse(k);
  keyPair = makeKeyPair(
    new Uint8Array(keyPair.publicKey.data),
    new Uint8Array(keyPair.secretKey.data),
  );
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

// Find the canister ID. Allow override from the url with "canister_id=1234.."
let canisterId = "{__canister_id}";
const maybeCid = window.location.search.match(/(?:\\?|&)canisterId=([0-9a-fA-Fa-zA-Z]+)(?:&|$)/);
if (maybeCid) {
  canisterId = maybeCid[1];
}

// Load index.js from the canister.
icHttpAgent.retrieveAsset(canisterId, "index.js")
  .then(content => {
    const indexJs = new TextDecoder().decode(content);
    const script = document.createElement("script");
    script.innerText = indexJs;
    document.head.appendChild(script);
  });
