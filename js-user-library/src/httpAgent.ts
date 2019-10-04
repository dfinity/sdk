import { Buffer } from "buffer/";
import { toHex } from "./buffer";
import * as cbor from "./cbor";
import { assertNever } from "./never";
import { makeNonce } from "./nonce";
import { requestIdOf } from "./requestId";

// TODO:
// * Handle errors everywhere we `await`

export type CanisterId = Buffer;
export type RequestId = Buffer;

// Common request fields.
export interface Request extends Record<string, any> {
  request_type: ReadRequestType | SubmitRequestType;
  // expiry?:;
  // NOTE: `nonce` is optional in the spec, but we require it so that requests
  // are unique and we avoid a bug in the client when the same request is
  // submitted more than once: https://dfinity.atlassian.net/browse/DFN-895
  nonce: Buffer;
  // sender:;
  // sender_pubkey: Buffer;
  // sender_sig: Buffer;
}

interface Response extends Record<string, any> {}


// An ADT that represents requests to the "read" endpoint.
type ReadRequest
  = QueryRequest
  | RequestStatusRequest;

// The types of values allowed in the `request_type` field for read requests.
enum ReadRequestType {
  Query = "query",
  RequestStatus = "request-status",
}

// Pattern match on a read request.
const matchReadRequest = (
  handlers: {
    query: (fields: QueryRequest) => any,
    requestStatus: (fields: RequestStatusRequest) => any,
  },
) => (
  request: ReadRequest,
): any => {
  switch (request.request_type) {
    case ReadRequestType.Query: {
      return handlers.query(request);
    }
    case ReadRequestType.RequestStatus: {
      return handlers.requestStatus(request);
    }
    default: {
      // Make the type checker enforce that our switch cases are exhaustive
      return assertNever(request);
    }
  }
};


// The fields in a "query" read request.
interface QueryRequest extends Request {
  request_type: ReadRequestType.Query;
  canister_id: CanisterId;
  method_name: string;
  arg: Buffer;
}

// An ADT that represents responses to a "query" read request.
export type QueryResponse
  = QueryResponseReplied
  | QueryResponseRejected;

interface QueryResponseReplied extends Response {
  status: QueryResponseStatus.Replied;
  reply: { arg: Buffer };
}

interface QueryResponseRejected extends Response {
  status: QueryResponseStatus.Rejected;
  reject_code: RejectCode;
  reject_message: string;
}

enum QueryResponseStatus {
  Replied = "replied",
  Rejected = "rejected",
}

// Pattern match on the response to a query request.
// TODO: matchQueryResponse

// Construct a "query" read request.
const makeQueryRequest = (
  config: Config,
  {
    methodName,
    arg,
  }: {
    methodName: string,
    arg: Buffer,
  },
): QueryRequest => ({
  request_type: ReadRequestType.Query,
  nonce: config.nonceFn(),
  canister_id: config.canisterId,
  method_name: methodName,
  arg,
});


// The fields in a "request-status" read request.
interface RequestStatusRequest extends Request {
  request_type: ReadRequestType.RequestStatus;
  request_id: RequestId;
}

// An ADT that represents responses to a "request-status" read request.
export type RequestStatusResponse
  = RequestStatusResponsePending
  | RequestStatusResponseReplied
  | RequestStatusResponseRejected
  | RequestStatusResponseUnknown;

interface RequestStatusResponsePending extends Response {
  status: RequestStatusResponseStatus.Pending;
}

interface RequestStatusResponseReplied extends Response {
  status: RequestStatusResponseStatus.Replied;
  reply: { arg: Buffer };
}

interface RequestStatusResponseRejected extends Response {
  status: RequestStatusResponseStatus.Rejected;
  reject_code: RejectCode;
  reject_message: string;
}

interface RequestStatusResponseUnknown extends Response {
  status: RequestStatusResponseStatus.Unknown;
}

export enum RequestStatusResponseStatus {
  Pending = "pending",
  Replied = "replied",
  Rejected = "rejected",
  Unknown = "unknown",
}

// Pattern match on the response to a "request-status" request.
export const matchRequestStatusResponse = (
  handlers: {
    pending: (fields: RequestStatusResponsePending) => any,
    replied: (fields: RequestStatusResponseReplied) => any,
    rejected: (fields: RequestStatusResponseRejected) => any,
    unknown: (fields: RequestStatusResponseUnknown) => any,
  },
) => (
  response: RequestStatusResponse,
): any => {
  switch (response.status) {
    case RequestStatusResponseStatus.Pending: {
      return handlers.pending(response);
    }
    case RequestStatusResponseStatus.Replied: {
      return handlers.replied(response);
    }
    case RequestStatusResponseStatus.Rejected: {
      return handlers.rejected(response);
    }
    case RequestStatusResponseStatus.Unknown: {
      return handlers.unknown(response);
    }
    default: {
      // Make the type checker enforce that our switch cases are exhaustive
      return assertNever(response);
    }
  }
};

// Construct a "request-status" read request.
const makeRequestStatusRequest = (
  config: Config,
  {
    requestId,
  }: {
    requestId: RequestId,
  },
): RequestStatusRequest => ({
  request_type: ReadRequestType.RequestStatus,
  nonce: config.nonceFn(),
  request_id: requestId,
});


enum RejectCode {
  SysFatal = 1,
  SysTransient = 2,
  DestinationInvalid = 3,
  CanisterReject = 4,
  CanisterError = 5,
}


// An ADT that represents requests to the "submit" endpoint.
type SubmitRequest
  = CallRequest;

// The types of values allowed in the `request_type` field for submit requests.
enum SubmitRequestType {
  Call = "call",
}

// Pattern match on a submit request.
const matchSubmitRequest = (
  handlers: {
    call: (fields: CallRequest) => any,
  },
) => (
  request: SubmitRequest,
): any => {
  switch (request.request_type) {
    case SubmitRequestType.Call: {
      return handlers.call(request);
    }
    default: {
      // Make the type checker enforce that our switch cases are exhaustive
      // FIXME: this causes a type error since we currently only have 1 tag
      // return assertNever(request);
    }
  }
};

// The fields in a "call" submit request.
interface CallRequest extends Request {
  request_type: SubmitRequestType.Call;
  canister_id: CanisterId;
  method_name: string;
  arg: Buffer;
}

// Construct a "call" submit request.
const makeCallRequest = (
  config: Config,
  {
    methodName,
    arg,
  }: {
    methodName: string,
    arg: Buffer,
  },
): CallRequest => ({
  request_type: SubmitRequestType.Call,
  nonce: config.nonceFn(),
  canister_id: config.canisterId,
  method_name: methodName,
  arg,
});


interface SubmitResponse extends Response {
  requestId: RequestId;
  response: Response;
}


const submit = (
  config: Config,
) => async (
  request: SubmitRequest,
): Promise<SubmitResponse> => {
  const body = cbor.encode(request);
  const response = await config.runFetch(Endpoint.Submit, body);
  const requestId = await requestIdOf(request);
  return { requestId, response };
};

const read = (
  config: Config,
) => async (
  request: ReadRequest,
): Promise<Response> => {
  const body = cbor.encode(request);
  return config.runFetch(Endpoint.Read, body);
};

const call = (
  config: Config,
) => async ({
  methodName,
  arg,
}: {
  methodName: string,
  arg: Buffer,
}): Promise<SubmitResponse> => {
  const request = makeCallRequest(config, {
    methodName,
    arg,
  });
  return submit(config)(request);
};

const requestStatus = (
  config: Config,
) => async ({
  requestId,
}: {
  requestId: RequestId,
}): Promise<RequestStatusResponse> => {
  const request = makeRequestStatusRequest(config, { requestId });
  const response = await read(config)(request);
  const body = Buffer.from(await response.arrayBuffer());
  return cbor.decode(body) as RequestStatusResponse;
};

const query = (
  config: Config,
) => async ({
  methodName,
  arg,
}: {
  methodName: string,
  arg: Buffer,
}): Promise<QueryResponse> => {
  const request = makeQueryRequest(config, {
    methodName,
    arg,
  });
  const response = await read(config)(request);
  const body = Buffer.from(await response.arrayBuffer());
  return cbor.decode(body) as QueryResponse;
};


const API_VERSION = "v1";

interface Options {
  canisterId: CanisterId;
  fetchFn?: WindowOrWorkerGlobalScope["fetch"];
  host?: string;
  nonceFn?: () => Buffer;
}

interface DefaultOptions {
  fetchFn: WindowOrWorkerGlobalScope["fetch"];
  host: string;
  nonceFn: () => Buffer;
}

const defaultOptions: DefaultOptions = {
  fetchFn: typeof window === "undefined" ? fetch : window.fetch.bind(window),
  host: "http://localhost:8000",
  nonceFn: makeNonce,
};


interface Config {
  canisterId: CanisterId;
  host: string;
  nonceFn: () => Buffer;
  runFetch(endpoint: Endpoint, body?: BodyInit | null): Promise<Response>;
}

const makeConfig = (options: Options): Config => {
  const withDefaults = { ...defaultOptions, ...options };
  return {
    ...withDefaults,
    runFetch: (endpoint, body) => {
      return withDefaults.fetchFn(`${withDefaults.host}/api/${API_VERSION}/${endpoint}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/cbor",
        },
        body,
      });
    },
  };
};


enum Endpoint {
  Read = "read",
  Submit = "submit",
}

export interface HttpAgent {
  call(fields: {
    methodName: string,
    arg: Buffer,
  }): Promise<SubmitResponse>;

  requestStatus(fields: {
    requestId: RequestId,
  }): Promise<RequestStatusResponse>;

  query(fields: {
    methodName: string,
    arg: Buffer,
  }): Promise<QueryResponse>;
}

export const makeHttpAgent = (options: Options): HttpAgent => {
  const config = makeConfig(options);
  return {
    call: call(config),
    requestStatus: requestStatus(config),
    query: query(config),
  };
};
