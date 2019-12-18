import { Buffer } from 'buffer/';
import { sign } from './auth';
import { BinaryBlob } from './blob';
import { CallRequest } from './call_request';
import { CanisterId } from './canisterId';
import * as cbor from './cbor';
import { makeNonce, Nonce } from './nonce';
import { QueryRequest } from './query_request';
import { QueryResponse } from './query_response';
import { ReadRequest } from './read_request';
import { ReadRequestType } from './read_request_type';
import { RequestId, requestIdOf } from './request_id';
import { RequestStatusRequest } from './request_status_request';
import { RequestStatusResponse } from './request_status_response';
import { Response } from './response';
import { SenderPubKey } from './sender_pub_key';
import { SenderSecretKey } from './sender_secret_key';
import { SenderSig } from './sender_sig';
import { SubmitRequest } from './submit_request';
import { SubmitRequestType } from './submit_request_type';
import { SubmitResponse } from './submit_response';

// A HTTP agent allows users to interact with a client of the internet computer
// using the available methods. It exposes an API that closely follows the
// public view of the internet computer, and is not intended to be exposed
// directly to the majority of users due to its low-level interface.
export function makeHttpAgent(options: Options): HttpAgent {
  const config = makeConfig(options);
  return {
    call: call(config),
    requestStatus: requestStatus(config),
    query: query(config),
  };
}

export interface HttpAgent {
  call(fields: { methodName: string; arg: BinaryBlob }): Promise<SubmitResponse>;

  query(fields: { methodName: string; arg: BinaryBlob }): Promise<QueryResponse>;

  requestStatus(fields: { requestId: RequestId }): Promise<RequestStatusResponse>;
}

// `Options` is the external representation of `Config` that allows us to
// provide optional fields with default values.
interface Options {
  canisterId: string;
  fetchFn?: WindowOrWorkerGlobalScope['fetch'];
  nonceFn?: () => Nonce;
  senderPubKey: SenderPubKey;
  senderSecretKey: SenderSecretKey;
  senderSigFn?: SigningConstructedFn;
}

type SigningConstructedFn = (secretKey: SenderSecretKey) => (requestId: RequestId) => SenderSig;

const defaultOptions: Partial<Options> = {
  fetchFn: typeof window === 'undefined' ? fetch : window.fetch.bind(window),
  nonceFn: makeNonce,
  senderSigFn: sign,
};

// `Config` is the internal representation of `Options`.
interface Config {
  canisterId: CanisterId;
  nonceFn: () => Nonce;
  senderPubKey: SenderPubKey;
  runFetch(endpoint: Endpoint, body?: BodyInit | null): Promise<Response>;
  senderSigFn(requestId: RequestId): SenderSig;
}

const API_VERSION = 'v1';

const makeConfig = (options: Options): Config => {
  const withDefaults = { ...defaultOptions, ...options } as Required<Options>;
  return {
    ...withDefaults,
    canisterId: CanisterId.fromHex(options.canisterId),
    // TODO We should be validating that this is the right public key.
    senderPubKey: options.senderPubKey,
    // If we set an override test function use that. Otherwise produce
    // a signing function.
    senderSigFn: withDefaults.senderSigFn(options.senderSecretKey),
    runFetch: (endpoint, body) => {
      return withDefaults.fetchFn(`/api/${API_VERSION}/${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/cbor',
        },
        body,
      });
    },
  };
};

enum Endpoint {
  Read = 'read',
  Submit = 'submit',
}

// Execute a read request
const read = (config: Config) => async (request: ReadRequest): Promise<Response> => {
  const body = cbor.encode(request);
  return config.runFetch(Endpoint.Read, body);
};

// Execute a submit request
const submit = (config: Config) => async (request: SubmitRequest): Promise<SubmitResponse> => {
  const body = cbor.encode(request);
  const response = await config.runFetch(Endpoint.Submit, body);
  const requestId = await requestIdOf(request);
  return { requestId, response };
};

// Execute a "call" request
const call = (config: Config) => async ({
  methodName,
  arg,
}: {
  methodName: string;
  arg: BinaryBlob;
}): Promise<SubmitResponse> => {
  const request = await makeCallRequest(config, {
    methodName,
    arg,
  });
  return submit(config)(request);
};

// Construct a "call" request.
const makeCallRequest = async (
  config: Config,
  {
    methodName,
    arg,
  }: {
    methodName: string;
    arg: BinaryBlob;
  },
): Promise<CallRequest> => {
  // TypeScript complains about `request_type` unless we manually add it to the
  // return value, even though it's already present.
  const requestType = SubmitRequestType.Call;
  const fields = {
    request_type: requestType,
    nonce: config.nonceFn(),
    canister_id: config.canisterId,
    method_name: methodName,
    arg,
  };
  const requestId = await requestIdOf(fields);
  return {
    ...fields,
    request_type: requestType,
    sender_pubkey: config.senderPubKey,
    sender_sig: config.senderSigFn(requestId),
  };
};

// Construct a query request
const makeQueryRequest = async (
  config: Config,
  {
    methodName,
    arg,
  }: {
    methodName: string;
    arg: BinaryBlob;
  },
): Promise<QueryRequest> => {
  // TypeScript complains about `request_type` unless we manually add it to the
  // return value, even though it's already present.
  const requestType = ReadRequestType.Query;
  const fields = {
    request_type: requestType,
    nonce: config.nonceFn(),
    canister_id: config.canisterId,
    method_name: methodName,
    arg,
  };
  const requestId = await requestIdOf(fields);
  return {
    ...fields,
    request_type: requestType,
    sender_pubkey: config.senderPubKey,
    sender_sig: config.senderSigFn(requestId),
  };
};

// Execute a query request
const query = (config: Config) => async ({
  methodName,
  arg,
}: {
  methodName: string;
  arg: BinaryBlob;
}): Promise<QueryResponse> => {
  const request = await makeQueryRequest(config, {
    methodName,
    arg,
  });
  const response = await read(config)(request);
  const body = Buffer.from(await response.arrayBuffer());
  return cbor.decode(body) as QueryResponse;
};

// Execute a request status request
const requestStatus = (config: Config) => async ({
  requestId,
}: {
  requestId: RequestId;
}): Promise<RequestStatusResponse> => {
  const request = await makeRequestStatusRequest(config, { requestId });
  const response = await read(config)(request);
  const body = Buffer.from(await response.arrayBuffer());
  return cbor.decode(body) as RequestStatusResponse;
};

// Construct a request status request
const makeRequestStatusRequest = async (
  config: Config,
  {
    requestId,
  }: {
    requestId: RequestId;
  },
): Promise<RequestStatusRequest> => {
  // TypeScript complains about `request_type` unless we manually add it to the
  // return value, even though it's already present.
  const requestType = ReadRequestType.RequestStatus;
  const fields = {
    request_type: requestType,
    nonce: config.nonceFn(),
    request_id: requestId,
  };
  const currentRequestId = await requestIdOf(fields);
  return {
    ...fields,
    request_type: requestType,
    sender_pubkey: config.senderPubKey,
    sender_sig: config.senderSigFn(currentRequestId),
  };
};
