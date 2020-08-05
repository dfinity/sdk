import { generateKeyPair, KeyPair, makeKeyPair, Principal } from '@dfinity/agent';
import localforage from 'localforage';
import * as storage from './storage';

const localStorageCanisterIdKey = 'dfinity-ic-canister-id';
const localStorageHostKey = 'dfinity-ic-host';
const localStorageIdentityKey = 'dfinity-ic-user-identity';
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

  const lsValue = await storage.retrieve(localStorageName);
  if (lsValue) {
    return lsValue;
  }

  return defaultValue;
}

function getCanisterId(s: string | undefined): Principal | undefined {
  if (s === undefined) {
    return undefined;
  } else {
    try {
      return Principal.fromText(s);
    } catch (_) {
      return undefined;
    }
  }
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
    const principal = await _getVariable('canisterId', localStorageCanisterIdKey);
    return new SiteInfo(
      DomainKind.Unknown,
      principal !== undefined ? Principal.fromText(principal) : undefined,
    );
  }

  public static async fromWindow(): Promise<SiteInfo> {
    const { hostname } = window.location;
    const components = hostname.split('.');
    const [maybeCId, maybeIc0, maybeApp] = components.slice(-3);
    const subdomain = components.slice(0, -3).join('.');

    if (maybeIc0 === 'ic0' && maybeApp === 'app') {
      return new SiteInfo(DomainKind.Ic0, getCanisterId(maybeCId), subdomain);
    } else if (maybeIc0 === 'lvh' && maybeApp === 'me') {
      return new SiteInfo(DomainKind.Lvh, getCanisterId(maybeCId), subdomain);
    } else if (maybeIc0 === 'localhost' && maybeApp === undefined) {
      /// Allow subdomain of localhost.
      return new SiteInfo(DomainKind.Localhost, getCanisterId(maybeCId), subdomain);
    } else if (maybeApp === 'localhost') {
      /// Allow subdomain of localhost, but maybeIc0 is the canister ID.
      return new SiteInfo(
        DomainKind.Localhost,
        getCanisterId(maybeIc0),
        `${maybeCId}.${subdomain}`,
      );
    } else {
      return this.unknown();
    }
  }

  private _isWorker = false;

  constructor(
    public readonly kind: DomainKind,
    public readonly principal?: Principal,
    public readonly subdomain = '',
  ) {}

  public async setLogin(username: string, password: string): Promise<void> {
    await this.store(localStorageLoginKey, JSON.stringify([username, password]));
  }

  public async getLogin(): Promise<[string, string] | undefined> {
    const maybeCreds = await this.retrieve(localStorageLoginKey);
    return maybeCreds !== undefined ? JSON.parse(maybeCreds) : undefined;
  }

  public async getKeyPair(): Promise<KeyPair> {
    let k = await _getVariable('userIdentity', localStorageIdentityKey);
    if (k === undefined) {
      k = await this.retrieve(localStorageIdentityKey);
    }

    if (k) {
      const kp = JSON.parse(k);
      return makeKeyPair(new Uint8Array(kp.publicKey.data), new Uint8Array(kp.secretKey.data));
    } else {
      const kp = generateKeyPair();
      await this.store(localStorageIdentityKey, JSON.stringify(kp));

      return kp;
    }
  }

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
        default:
          return host || '';
      }
    }
  }

  private async store(name: string, value: string): Promise<void> {
    await localforage.setItem(name, value);
    await storage.store(name, value);
  }

  private async retrieve(name: string): Promise<string | undefined> {
    const maybeValue = await storage.retrieve(name);
    if (maybeValue === undefined) {
      return localforage.getItem<string>(name);
    } else {
      return maybeValue;
    }
  }
}
