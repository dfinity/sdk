import {
  HttpAgent,
  generateKeyPair,
  makeAuthTransform,
  makeKeyPair,
  makeNonceTransform,
  IDL,
} from '../out';

const localStorageIdentityKey = 'dfinity-ic-user-identity';
const localStorageCanisterIdKey = 'dfinity-ic-canister-id';
const localStorageHostKey = 'dfinity-ic-host';

function _getVariable(queryName, localStorageName, defaultValue) {
  const queryValue = window.location.search.match(new RegExp(`[?&]${queryName}=([^&]*)(?:&|$)`));
  if (queryValue) {
    return decodeURIComponent(queryValue[1]);
  }
  const lsValue = window.localStorage.getItem(localStorageName);
  if (lsValue) {
    return lsValue;
  }
  return defaultValue;
}

// Retrieve and execute a JavaScript file from the server.
function _loadJs(canisterId, filename) {
  return icHttpAgent.retrieveAsset(canisterId, filename)
    .then(content => {
      const js = new TextDecoder().decode(content);
      const dataUri = 'data:text/javascript;base64,' + btoa(js);
      return import(/* webpackIgnore: true */dataUri);
    });
}

let k = _getVariable('userIdentity', localStorageIdentityKey);
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
  window.localStorage.setItem(localStorageIdentityKey, JSON.stringify(keyPair));
}

// Figure out the host.
let host = _getVariable('host', localStorageHostKey, '');
if (host) {
  try {
    host = JSON.parse(host);
  } catch (_) {
  }

  if (Array.isArray(host)) {
    host = '' + host[(Math.random() * host.length)| 0];
  } else {
    host = '' + host;
  }
}

const agent = new HttpAgent({ host });
agent.addTransform(makeNonceTransform());
agent.addTransform(makeAuthTransform(keyPair));

window.icHttpAgent = agent;

// Find the canister ID. Allow override from the url with 'canister_id=1234..'
let canisterId = _getVariable('canisterId', localStorageCanisterIdKey, '');
if (!canisterId) {
  // Show an error.
  const div = document.createElement('div');
  div.innerText = 'Could not find the canister ID to use. Please provide one in the query parameters.';
  document.body.replaceChild(div, document.body.getElementsByTagName('app').item(0));
} else {
  if (window.location.pathname == '/candid') {
    // Load candid.js from the canister.
    _loadJs(canisterId, 'candid.js')
      .then(candid => {
        const canister = icHttpAgent.makeActorFactory(candid.default)({ canisterId });
        return import('./candid/candid.js').then(render => {
          const actor = candid.default({IDL});
          render.render(canisterId, actor, canister);
        });
      })
      .catch(err => {
        const div = document.createElement('div');
        div.innerText = 'An error happened while loading candid:';
        const pre = document.createElement('pre');
        pre.innerHTML = err.stack;
        div.appendChild(pre);
        document.body.replaceChild(div, document.body.getElementsByTagName('app').item(0));
      });
  } else {
    // Load index.js from the canister.
    setTimeout(() => {
      _loadJs(canisterId, 'index.js')
          .catch(err => {
            const div = document.createElement('div');
            div.innerText = 'An error happened while loading the canister:';
            const pre = document.createElement('pre');
            pre.innerHTML = err.stack;
            div.appendChild(pre);
            document.body.replaceChild(div, document.body.getElementsByTagName('app').item(0));
          });
      }, 0);
  }
}
