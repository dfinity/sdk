import { Buffer } from 'buffer/';
import { ActorFactory } from '../actor';
import * as actor from '../actor';
import { Agent } from '../agent';
import * as cbor from '../cbor';
import {
  AuthHttpAgentRequestTransformFn,
  Endpoint,
  HttpAgentReadRequest,
  HttpAgentRequest,
  HttpAgentRequestTransformFn,
  HttpAgentSubmitRequest,
  QueryFields,
  QueryResponse,
  ReadRequest,
  ReadRequestType,
  ReadResponse,
  RequestStatusFields,
  RequestStatusResponse,
  SignedHttpAgentRequest,
  SubmitRequest,
  SubmitRequestType,
  SubmitResponse,
} from '../http_agent_types';
import * as IDL from '../idl';
import { Principal } from '../principal';
import { requestIdOf } from '../request_id';
import { BinaryBlob, blobFromHex, JsonObject } from '../types';

const API_VERSION = 'v1';

// HttpAgent options that can be used at construction.
export interface HttpAgentOptions {
  // Another HttpAgent to inherit configuration (pipeline and fetch) of. This
  // is only used at construction.
  source?: HttpAgent;

  // A surrogate to the global fetch function. Useful for testing.
  fetch?: typeof fetch;

  // The host to use for the client. By default, uses the same host as
  // the current page.
  host?: string;

  // The principal used to send messages. This cannot be empty at the request
  // time (will throw).
  principal?: Principal | Promise<Principal>;

  credentials?: {
    name: string;
    password?: string;
  };
}

declare const window: Window & { fetch: typeof fetch };
declare const global: { fetch: typeof fetch };
declare const self: { fetch: typeof fetch };

function getDefaultFetch(): typeof fetch {
  const result =
    typeof window === 'undefined'
      ? typeof global === 'undefined'
        ? typeof self === 'undefined'
          ? undefined
          : self.fetch.bind(self)
        : global.fetch.bind(global)
      : window.fetch.bind(window);

  if (!result) {
    throw new Error('Could not find default `fetch` implementation.');
  }

  return result;
}

// A HTTP agent allows users to interact with a client of the internet computer
// using the available methods. It exposes an API that closely follows the
// public view of the internet computer, and is not intended to be exposed
// directly to the majority of users due to its low-level interface.
//
// There is a pipeline to apply transformations to the request before sending
// it to the client. This is to decouple signature, nonce generation and
// other computations so that this class can stay as simple as possible while
// allowing extensions.
export class HttpAgent implements Agent {
  private readonly _pipeline: HttpAgentRequestTransformFn[] = [];
  private _authTransform: AuthHttpAgentRequestTransformFn | null = null;
  private readonly _fetch: typeof fetch;
  private readonly _host: URL;
  private readonly _principal: Promise<Principal> | null = null;
  private readonly _credentials: string | undefined;

  constructor(options: HttpAgentOptions = {}) {
    if (options.source) {
      this._pipeline = [...options.source._pipeline];
      this._authTransform = options.source._authTransform;
      this._principal = options.source._principal;
    }
    this._fetch = options.fetch || getDefaultFetch() || fetch.bind(global);
    if (options.host) {
      if (!options.host.match(/^[a-z]+:/) && typeof window !== 'undefined') {
        this._host = new URL(window.location.protocol + '//' + options.host);
      } else {
        this._host = new URL(options.host);
      }
    } else {
      const location = window?.location;
      if (!location) {
        throw new Error('Must specify a host to connect to.');
      }
      this._host = new URL(location + '');
    }
    if (options.principal) {
      this._principal = Promise.resolve(options.principal);
    }
    if (options.credentials) {
      const { name, password } = options.credentials;
      this._credentials = `${name}${password ? ':' + password : ''}`;
    }
  }

  public addTransform(fn: HttpAgentRequestTransformFn, priority = fn.priority || 0) {
    // Keep the pipeline sorted at all time, by priority.
    const i = this._pipeline.findIndex(x => (x.priority || 0) < priority);
    this._pipeline.splice(i >= 0 ? i : this._pipeline.length, 0, Object.assign(fn, { priority }));
  }

  public setAuthTransform(fn: AuthHttpAgentRequestTransformFn) {
    this._authTransform = fn;
  }

  public async call(
    canisterId: Principal | string,
    fields: {
      methodName: string;
      arg: BinaryBlob;
    },
    principal?: Principal | Promise<Principal>,
  ): Promise<SubmitResponse> {
    let p = this._principal || principal;
    if (!p) {
      throw new Error('No principal specified.');
    }
    p = await Promise.resolve(p);

    return this.submit({
      request_type: SubmitRequestType.Call,
      canister_id: typeof canisterId === 'string' ? Principal.fromText(canisterId) : canisterId,
      method_name: fields.methodName,
      arg: fields.arg,
      sender: p.toBlob(),
      ingress_expiry: 300,
    });
  }

  public async install(
    canisterId: Principal | string,
    fields: {
      module: BinaryBlob;
      arg?: BinaryBlob;
    },
    principal?: Principal,
  ): Promise<SubmitResponse> {
    let p = this._principal || principal;
    if (!p) {
      throw new Error('No principal specified.');
    }
    p = await Promise.resolve(p);

    return this.submit({
      request_type: SubmitRequestType.InstallCode,
      canister_id: typeof canisterId === 'string' ? Principal.fromText(canisterId) : canisterId,
      module: fields.module,
      arg: fields.arg || blobFromHex(''),
      sender: p.toBlob(),
      ingress_expiry: 300,
    });
  }

  public async createCanister(principal?: Principal): Promise<SubmitResponse> {
    let p = this._principal || principal;
    if (!p) {
      throw new Error('No principal specified.');
    }
    p = await Promise.resolve(p);

    return this.submit({
      request_type: SubmitRequestType.CreateCanister,
      sender: p.toBlob(),
      ingress_expiry: 300,
    });
  }

  public async query(
    canisterId: Principal | string,
    fields: QueryFields,
    principal?: Principal,
  ): Promise<QueryResponse> {
    let p = this._principal || principal;
    if (!p) {
      throw new Error('No principal specified.');
    }
    p = await Promise.resolve(p);

    return this.read({
      request_type: ReadRequestType.Query,
      canister_id: typeof canisterId === 'string' ? Principal.fromText(canisterId) : canisterId,
      method_name: fields.methodName,
      arg: fields.arg,
      sender: p.toBlob(),
      ingress_expiry: 300,
    }) as Promise<QueryResponse>;
  }

  public async requestStatus(
    fields: RequestStatusFields,
    principal?: Principal,
  ): Promise<RequestStatusResponse> {
    let p = this._principal || principal;
    if (!p) {
      throw new Error('No principal specified.');
    }
    p = await Promise.resolve(p);

    return this.read({
      request_type: ReadRequestType.RequestStatus,
      request_id: fields.requestId,
      ingress_expiry: 300,
    }) as Promise<RequestStatusResponse>;
  }

  public async status(): Promise<JsonObject> {
    const headers: Record<string, string> = this._credentials
      ? {
          Authorization: 'Basic ' + btoa(this._credentials),
        }
      : {};

    const response = await this._fetch(
      '' + new URL(`/api/${API_VERSION}/${Endpoint.Status}`, this._host),
      { headers },
    );

    if (!response.ok) {
      throw new Error(
        `Server returned an error:\n` +
          `  Code: ${response.status} (${response.statusText}\n)` +
          `  Body: ${await response.text()}\n`,
      );
    }

    const buffer = await response.arrayBuffer();
    return cbor.decode(new Uint8Array(buffer));
  }

  public makeActorFactory(actorInterfaceFactory: IDL.InterfaceFactory): ActorFactory {
    return actor.makeActorFactory(actorInterfaceFactory);
  }

  protected _transform(
    request: HttpAgentRequest,
  ): Promise<HttpAgentRequest | SignedHttpAgentRequest> {
    let p = Promise.resolve(request);

    for (const fn of this._pipeline) {
      p = p.then(r => fn(r).then(r2 => r2 || r));
    }

    if (this._authTransform != null) {
      return p.then(this._authTransform);
    } else {
      return p;
    }
  }

  protected async submit(submit: SubmitRequest): Promise<SubmitResponse> {
    const transformedRequest = (await this._transform({
      request: {
        body: null,
        method: 'POST',
        headers: {
          'Content-Type': 'application/cbor',
          ...(this._credentials ? { Authorization: 'Basic ' + btoa(this._credentials) } : {}),
        },
      },
      endpoint: Endpoint.Submit,
      body: submit,
    })) as HttpAgentSubmitRequest;

    const body = cbor.encode(transformedRequest.body);

    // Run both in parallel. The fetch is quite expensive, so we have plenty of time to
    // calculate the requestId locally.
    const [response, requestId] = await Promise.all([
      this._fetch('' + new URL(`/api/${API_VERSION}/${Endpoint.Submit}`, this._host), {
        ...transformedRequest.request,
        body,
      }),
      requestIdOf(submit),
    ]);

    if (!response.ok) {
      throw new Error(
        `Server returned an error:\n` +
          `  Code: ${response.status} (${response.statusText}\n)` +
          `  Body: ${await response.text()}\n`,
      );
    }

    return {
      requestId,
      response: {
        ok: response.ok,
        status: response.status,
        statusText: response.statusText,
      },
    };
  }

  protected async read(request: ReadRequest): Promise<ReadResponse> {
    const transformedRequest = (await this._transform({
      request: {
        method: 'POST',
        headers: {
          'Content-Type': 'application/cbor',
          ...(this._credentials ? { Authorization: 'Basic ' + btoa(this._credentials) } : {}),
        },
      },
      endpoint: Endpoint.Read,
      body: request,
    })) as HttpAgentReadRequest;

    const body = cbor.encode(transformedRequest.body);

    const response = await this._fetch(
      '' + new URL(`/api/${API_VERSION}/${Endpoint.Read}`, this._host),
      {
        ...transformedRequest.request,
        body,
      },
    );

    if (!response.ok) {
      throw new Error(
        `Server returned an error:\n` +
          `  Code: ${response.status} (${response.statusText}\n)` +
          `  Body: ${await response.text()}\n`,
      );
    }

    return cbor.decode(Buffer.from(await response.arrayBuffer()));
  }
}
