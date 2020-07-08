import {
  Agent,
  CanisterId,
  generateKeyPair,
  HttpAgent,
  KeyPair,
  makeAuthTransform,
  makeKeyPair,
  makeNonceTransform,
  Principal,
  ProxyAgent,
  ProxyMessage,
} from '@dfinity/agent';
import localforage from 'localforage';

const localStorageIdentityKey = 'dfinity-ic-user-identity';
const localStorageCanisterIdKey = 'dfinity-ic-canister-id';
const localStorageHostKey = 'dfinity-ic-host';
const localStorageLoginKey = 'dfinity-ic-login';

async function _getVariable(name: string, localStorageName: string): Promise<string | undefined>;
async function _getVariable(
  name: string,
  localStorageName: string,
  defaultValue: string,
): Promise<string>;
async function _getVariable(
  name: string,
  localStorageName: string,
  defaultValue?: string,
): Promise<string | undefined> {
  const params = new URLSearchParams(window.location.search);

  const maybeValue = params.get(name);
  if (maybeValue) {
    return maybeValue;
  }

  const lsValue = await localforage.getItem<string>(localStorageName);
  if (lsValue) {
    return lsValue;
  }

  return defaultValue;
}

export async function setLogin(username: string, password: string): Promise<void> {
  await localforage.setItem(localStorageLoginKey, JSON.stringify([username, password]));
}

export async function getLogin(): Promise<[string, string] | undefined> {
  const maybeCreds = await localforage.getItem<string>(localStorageLoginKey);
  return maybeCreds !== undefined ? JSON.parse(maybeCreds) : undefined;
}

export enum DomainKind {
  Unknown,
  Localhost,
  Ic0,
  Lvh,
}

export class SiteInfo {
  public static async worker(): Promise<SiteInfo> {
    const siteInfo = await SiteInfo.fromWindow();
    siteInfo._isWorker = true;

    return siteInfo;
  }

  public static async unknown(): Promise<SiteInfo> {
    const canisterId = await _getVariable('canisterId', localStorageCanisterIdKey);
    return new SiteInfo(
      DomainKind.Unknown,
      canisterId !== undefined ? CanisterId.fromText(canisterId) : undefined,
    );
  }

  public static async fromWindow(): Promise<SiteInfo> {
    const { hostname } = window.location;
    const components = hostname.split('.');
    const [maybeCId, maybeIc0, maybeApp] = components.slice(-3);
    const subdomain = components.slice(0, -3).join('.');

    if (maybeIc0 === 'ic0' && maybeApp === 'app') {
      return new SiteInfo(DomainKind.Ic0, CanisterId.fromHexWithChecksum(maybeCId), subdomain);
    } else if (maybeIc0 === 'lvh' && maybeApp === 'me') {
      return new SiteInfo(DomainKind.Lvh, CanisterId.fromHexWithChecksum(maybeCId), subdomain);
    } else if (maybeIc0 === 'localhost' && maybeApp === undefined) {
      /// Allow subdomain of localhost.
      return new SiteInfo(
        DomainKind.Localhost,
        CanisterId.fromHexWithChecksum(maybeCId),
        subdomain,
      );
    } else if (maybeApp === 'localhost') {
      /// Allow subdomain of localhost, but maybeIc0 is the canister ID.
      return new SiteInfo(
        DomainKind.Localhost,
        CanisterId.fromHexWithChecksum(maybeIc0),
        `${maybeCId}.${subdomain}`,
      );
    } else {
      return this.unknown();
    }
  }

  private _isWorker = false;

  constructor(
    public readonly kind: DomainKind,
    public readonly canisterId?: CanisterId,
    public readonly subdomain = '',
  ) {}

  public isUnknown() {
    return this.kind === DomainKind.Unknown;
  }

  public async getWorkerHost(): Promise<string> {
    if (this._isWorker) {
      return '';
    }

    const { port, protocol } = window.location;

    switch (this.kind) {
      case DomainKind.Unknown:
        throw new Error('Cannot get worker host inside a worker.');
      case DomainKind.Ic0:
        return `${protocol}//z.ic0.app${port ? ':' + port : ''}`;
      case DomainKind.Lvh:
        return `${protocol}//z.lvh.me${port ? ':' + port : ''}`;
      case DomainKind.Localhost:
        return `${protocol}//z.localhost${port ? ':' + port : ''}`;
    }
  }

  public async getHost(): Promise<string> {
    // Figure out the host.
    let host = await _getVariable('host', localStorageHostKey, '');

    if (host) {
      try {
        host = JSON.parse(host);

        if (Array.isArray(host)) {
          return '' + host[Math.floor(Math.random() * host.length)];
        } else {
          return '' + host;
        }
      } catch (_) {
        return host;
      }
    } else {
      const { port, protocol } = window.location;

      switch (this.kind) {
        case DomainKind.Unknown:
          return '';
        case DomainKind.Ic0:
          // TODO: think if we want to have this hard coded here. We might.
          return `${protocol}//gw.dfinity.network${port ? ':' + port : ''}`;
        case DomainKind.Lvh:
          return `${protocol}//r.lvh.me${port ? ':' + port : ''}`;
        case DomainKind.Localhost:
          return `${protocol}//r.localhost${port ? ':' + port : ''}`;
      }
    }

    return host || '';
  }
}

async function getKeyPair(forceNewPair = false): Promise<KeyPair> {
  const k = forceNewPair ? null : await _getVariable('userIdentity', localStorageIdentityKey);
  let keyPair: KeyPair;

  if (k) {
    const kp = JSON.parse(k);
    keyPair = makeKeyPair(new Uint8Array(kp.publicKey.data), new Uint8Array(kp.secretKey.data));
  } else {
    const kp = generateKeyPair();
    // TODO(eftycis): use a parser+an appropriate format to avoid
    // leaking the key when constructing the string for
    // localStorage.
    if (!forceNewPair) {
      await localforage.setItem(localStorageIdentityKey, JSON.stringify(kp));
    }

    keyPair = kp;
  }

  return keyPair;
}

export async function createAgent(site: SiteInfo): Promise<Agent> {
  const workerHost = site.isUnknown() ? undefined : await site.getWorkerHost();
  const host = await site.getHost();

  if (!workerHost) {
    const keyPair = await getKeyPair();
    const principal = Principal.selfAuthenticating(keyPair.publicKey);
    const creds = await getLogin();
    const agent = new HttpAgent({
      host,
      principal,
      ...(creds && { credentials: { name: creds[0], password: creds[1] } }),
    });
    agent.addTransform(makeNonceTransform());
    agent.setAuthTransform(makeAuthTransform(keyPair));

    return agent;
  } else {
    return createWorkerAgent(site, workerHost, host);
  }
}

async function createWorkerAgent(site: SiteInfo, workerHost: string, host: string): Promise<Agent> {
  // Create the IFRAME.
  let messageQueue: ProxyMessage[] | null = [];
  let loaded = false;
  const agent = new ProxyAgent((msg: ProxyMessage) => {
    if (!loaded) {
      if (!messageQueue) {
        throw new Error('No Message Queue but need Queueing...');
      }
      messageQueue.push(msg);
    } else {
      iframeEl.contentWindow!.postMessage(msg, '*');
    }
  });

  const iframeEl = document.createElement('iframe');

  iframeEl.src = `${workerHost}/worker.html?${host ? 'host=' + encodeURIComponent(host) : ''}`;
  window.addEventListener('message', ev => {
    if (ev.origin === workerHost) {
      switch (ev.data) {
        case 'ready':
          const q = messageQueue?.splice(0, messageQueue.length) || [];
          for (const msg of q) {
            iframeEl.contentWindow!.postMessage(msg, workerHost);
          }

          loaded = true;
          messageQueue = null;
          break;

        case 'login':
          const url = new URL(workerHost);
          url.pathname = '/login.html';
          url.searchParams.append('redirect', '' + window.location);
          window.location.replace('' + url);
          break;

        default:
          if (typeof ev.data === 'object') {
            agent.onmessage(ev.data);
          } else {
            throw new Error('Invalid message from worker: ' + JSON.stringify(ev.data));
          }
      }
    }
  });

  document.head.append(iframeEl);
  return agent;
}
