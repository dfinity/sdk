import {
  generateKeyPair,
  HttpAgent,
  IDL,
  makeAuthTransform,
  makeKeyPair,
  makeNonceTransform,
} from '@internet-computer/userlib';

interface WindowWithInternetComputer extends Window {
  icHttpAgent: HttpAgent;
  ic: {
    httpAgent: HttpAgent;
  };
}
declare const window: WindowWithInternetComputer;

const localStorageIdentityKey = 'dfinity-ic-user-identity';
const localStorageCanisterIdKey = 'dfinity-ic-canister-id';
const localStorageHostKey = 'dfinity-ic-host';

function _getVariable(
  queryName: string,
  localStorageName: string,
  defaultValue?: string,
): string | undefined {
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
async function _loadJs(canisterId: string, filename: string): Promise<any> {
  const content = await window.icHttpAgent.retrieveAsset(canisterId, filename);
  const js = new TextDecoder().decode(content);
  const dataUri = 'data:text/javascript;base64,' + btoa(js);
  // TODO(hansl): either get rid of eval, or rid of webpack, or make this
  // work without this horrible hack.
  return eval('import("' + dataUri + '")'); // tslint:disable-line
}

const k = _getVariable('userIdentity', localStorageIdentityKey);
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

    if (Array.isArray(host)) {
      host = '' + host[Math.floor(Math.random() * host.length)];
    } else {
      host = '' + host;
    }
  } catch (_) {
    host = '';
  }
}

const agent = new HttpAgent({ host });
agent.addTransform(makeNonceTransform());
agent.addTransform(makeAuthTransform(keyPair));

window.icHttpAgent = agent;
window.ic = { httpAgent: agent };

async function _main() {
  // Find the canister ID. Allow override from the url with 'canister_id=1234..'
  const canisterId = _getVariable('canisterId', localStorageCanisterIdKey, '');
  if (!canisterId) {
    // Show an error.
    const div = document.createElement('div');
    div.innerText =
      'Could not find the canister ID to use. Please provide one in the query parameters.';

    document.body.replaceChild(div, document.body.getElementsByTagName('app').item(0)!);
  } else {
    if (window.location.pathname === '/candid') {
      // Load candid.js from the canister.
      const candid = await _loadJs(canisterId, 'candid.js');
      const canister = window.icHttpAgent.makeActorFactory(candid.default)({ canisterId });
      // @ts-ignore: Could not find a declaration file for module
      const render: any = await import(/* webpackIgnore: true */ './candid/candid.js');
      const actor = candid.default({ IDL });
      render.render(canisterId, actor, canister);
    } else {
      // Load index.js from the canister and execute it.
      await _loadJs(canisterId, 'index.js');
    }
  }
}

_main().catch(err => {
  const div = document.createElement('div');
  div.innerText = 'An error happened:';
  const pre = document.createElement('pre');
  pre.innerHTML = err.stack;
  div.appendChild(pre);
  document.body.replaceChild(div, document.body.getElementsByTagName('app').item(0)!);
});
