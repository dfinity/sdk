import { toByteArray } from 'base64-js';
import { Buffer } from 'buffer/';
import * as actor from './actor';
import { CanisterId } from './canisterId';
import * as cbor from './cbor';
import {
  Endpoint,
  HttpAgentReadRequest,
  HttpAgentRequest,
  HttpAgentRequestTransformFn,
  HttpAgentSubmitRequest,
  QueryFields,
  QueryResponse,
  QueryResponseStatus,
  ReadRequest,
  ReadRequestType,
  ReadResponse,
  RequestStatusResponse,
  ResponseStatusFields,
  SubmitRequest,
  SubmitRequestType,
  SubmitResponse,
} from './http_agent_types';
import * as IDL from './idl';
import { requestIdOf } from './request_id';
import { BinaryBlob } from './types';
const API_VERSION = 'v1';

// HttpAgent options that can be used at construction.
export interface HttpAgentOptions {
  // A parent to inherit configuration (pipeline and fetch) of. This is only
  // used at construction; if the parent is changed we don't propagate those
  // changes to the children.
  parent?: HttpAgent;

  // A surrogate to the global fetch function. Useful for testing.
  fetch?: typeof fetch;

  // The host to use for the client. By default, uses the same host as
  // the current page.
  host?: string;
}

declare const window: Window & { fetch: typeof fetch };
declare const global: { fetch: typeof fetch };
declare const self: { fetch: typeof fetch };

function getDefaultFetch() {
  return typeof window === 'undefined'
    ? typeof global === 'undefined'
      ? typeof self === 'undefined'
        ? undefined
        : self.fetch.bind(self)
      : global.fetch.bind(global)
    : window.fetch.bind(window);
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
export class HttpAgent {
  private readonly _pipeline: HttpAgentRequestTransformFn[] = [];
  private readonly _fetch: typeof fetch;
  private readonly _host: string = '';

  constructor(options: HttpAgentOptions = {}) {
    if (options.parent) {
      this._pipeline = [...options.parent._pipeline];
    }
    this._fetch = options.fetch || getDefaultFetch() || fetch.bind(global);
    if (options.host) {
      if (!options.host.match(/^[a-z]+:/) && typeof window !== 'undefined') {
        this._host = window.location.protocol + '//' + options.host;
      } else {
        this._host = options.host;
      }
    }
  }

  public addTransform(fn: HttpAgentRequestTransformFn, priority = fn.priority || 0) {
    // Keep the pipeline sorted at all time, by priotity.
    const i = this._pipeline.findIndex(x => (x.priority || 0) < priority);
    this._pipeline.splice(i >= 0 ? i : this._pipeline.length, 0, Object.assign(fn, { priority }));
  }

  public async submit(submit: SubmitRequest): Promise<SubmitResponse> {
    const transformedRequest = (await this._transform({
      request: {
        body: null,
        method: 'POST',
        headers: {
          'Content-Type': 'application/cbor',
        },
      },
      endpoint: Endpoint.Submit,
      body: submit,
    })) as HttpAgentSubmitRequest;

    const body = cbor.encode(transformedRequest.body);

    // Run both in parallel. The fetch is quite expensive, so we have plenty of time to
    // calculate the requestId locally.
    const [response, requestId] = await Promise.all([
      this._fetch(`${this._host}/api/${API_VERSION}/${Endpoint.Submit}`, {
        ...transformedRequest.request,
        body,
      }),
      requestIdOf(submit),
    ]);

    return { requestId, response };
  }

  public async read(request: ReadRequest): Promise<ReadResponse> {
    const transformedRequest = (await this._transform({
      request: {
        method: 'POST',
        headers: {
          'Content-Type': 'application/cbor',
        },
      },
      endpoint: Endpoint.Read,
      body: request,
    })) as HttpAgentReadRequest;

    const body = cbor.encode(transformedRequest.body);

    const response = await this._fetch(`${this._host}/api/${API_VERSION}/${Endpoint.Read}`, {
      ...transformedRequest.request,
      body,
    });
    return cbor.decode(Buffer.from(await response.arrayBuffer()));
  }

  public call(
    canisterId: CanisterId | string,
    fields: {
      methodName: string;
      arg: BinaryBlob;
    },
  ): Promise<SubmitResponse> {
    return this.submit({
      request_type: SubmitRequestType.Call,
      canister_id: typeof canisterId === 'string' ? new CanisterId(canisterId) : canisterId,
      method_name: fields.methodName,
      arg: fields.arg,
    });
  }

  public query(canisterId: CanisterId | string, fields: QueryFields): Promise<QueryResponse> {
    return this.read({
      request_type: ReadRequestType.Query,
      canister_id: typeof canisterId === 'string' ? new CanisterId(canisterId) : canisterId,
      method_name: fields.methodName,
      arg: fields.arg,
    }) as Promise<QueryResponse>;
  }

  public retrieveAsset(canisterId: CanisterId | string, path: string): Promise<Uint8Array> {
    const arg = IDL.encode([IDL.Text], [path]) as BinaryBlob;
    return this.query(canisterId, { methodName: '__dfx_asset_path', arg }).then(response => {
      switch (response.status) {
        case QueryResponseStatus.Rejected:
          throw new Error(`An error happened while retrieving asset "${path}".`);
        case QueryResponseStatus.Replied:
          const [content] = IDL.decode([IDL.Text], response.reply.arg);
          if (path.match(/\.(js|html|css)$/)) {
            return new TextEncoder().encode('' + content);
          } else {
            return toByteArray('' + content);
          }
      }
    });
  }

  public requestStatus(fields: ResponseStatusFields): Promise<RequestStatusResponse> {
    return this.read({
      request_type: ReadRequestType.RequestStatus,
      request_id: fields.requestId,
    }) as Promise<RequestStatusResponse>;
  }

  public get makeActorFactory() {
    return actor.makeActorFactory;
  }

  protected _transform(request: HttpAgentRequest): Promise<HttpAgentRequest> {
    let p = Promise.resolve(request);

    for (const fn of this._pipeline) {
      p = p.then(r => fn(r).then(r2 => r2 || r));
    }

    return p;
  }
}
