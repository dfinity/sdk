import { assertNever } from "./never";

// Common request fields.
interface Request {
  request_type: ReadRequestType | SubmitRequestType;
  // expiry?:;
  // NOTE: `nonce` is optional in the spec, but we should probably provide it
  // nonce: Blob;
  // sender:;
  // sender_pubkey: Blob;
  // sender_sig: Blob;
}


// An ADT that represents requests to the "read" endpoint.
type ReadRequest
  = ReadQueryRequest
  | ReadRequestStatusRequest;

// The types of values allowed in the `request_type` field for read requests.
enum ReadRequestType {
  query = "query",
  requestStatus = "request-status",
}

// Pattern match on a read request.
const matchReadRequest = (
  handlers: {
    query: (x: ReadQueryRequest) => any,
    requestStatus: (x: ReadRequestStatusRequest) => any,
  },
) => (
  request: ReadRequest,
): any => {
  switch (request.request_type) {
    case ReadRequestType.query: {
      return handlers.query(request);
    }
    case ReadRequestType.requestStatus: {
      return handlers.requestStatus(request);
    }
    default: {
      // Make the type checker enforce that our switch cases are exhaustive
      return assertNever(request);
    }
  }
};

// The fields in a "query" read request.
interface ReadQueryRequest extends Request {
  request_type: ReadRequestType.query;
  canister_id: number;
  method_name: string;
  arg: Blob;
}

// The fields in a "request-status" read request.
interface ReadRequestStatusRequest extends Request {
  request_type: ReadRequestType.requestStatus;
  request_id: number;
}

// Construct a "query" read request.
const readQuery = ({
  canisterId,
  methodName,
  arg,
}: {
  canisterId: number,
  methodName: string,
  arg: Blob,
}): ReadQueryRequest => ({
  request_type: ReadRequestType.query,
  canister_id: canisterId,
  method_name: methodName,
  arg,
});


// Construct a "request-status" read request.
const readRequestStatus = ({
  requestId,
}: {
  requestId: number,
}): ReadRequestStatusRequest => ({
  request_type: ReadRequestType.requestStatus,
  request_id: requestId,
});

export enum ReadRequestStatusResponseStatus {
  unknown,
  pending,
  replied,
  rejected,
}


enum RejectCode {
  SysFatal = 1,
  SysTransient = 2,
  DestinationInvalid = 3,
  CanisterReject = 4,
  CanisterError = 5,
}


// An ADT that represents requests to the "submit" endpoint.
type SubmitRequest
  = SubmitCallRequest;

// The types of values allowed in the `request_type` field for submit requests.
enum SubmitRequestType {
  call = "call",
}

// Pattern match on a submit request.
const matchSubmitRequest = (
  handlers: {
    call: (x: SubmitCallRequest) => any,
  },
) => (
  request: SubmitRequest,
): any => {
  switch (request.request_type) {
    case SubmitRequestType.call: {
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
interface SubmitCallRequest extends Request {
  request_type: SubmitRequestType.call;
  canister_id: number;
  method_name: string;
  arg: Blob;
}

// Construct a "call" submit request.
const submitCall = ({
  canisterId,
  methodName,
  arg,
}: {
  canisterId: number,
  methodName: string,
  arg: Blob,
}): SubmitCallRequest => ({
  request_type: SubmitRequestType.call,
  canister_id: canisterId,
  method_name: methodName,
  arg,
});


interface SubmitResponse {
  requestId: number;
  response: Response;
}


const submit = (
  config: Config,
) => async (
  request: SubmitRequest,
): Promise<SubmitResponse> => {
  const body = matchSubmitRequest({
    call: (fields) => {
      // FIXME: convert `fields` to `body`
      return "FIXME: call";
    },
  })(request);
  // TODO: decode body from CBOR
  const response = await config.runFetch(Endpoint.submit, body);
  return {
    requestId: -1, // FIXME
    response,
  };
};

const read = (
  config: Config,
) => async (
  request: ReadRequest,
): Promise<Response> => {
  const body = matchReadRequest({
    query: (fields) => {
      return "FIXME: query"; // FIXME: CBOR
    },
    requestStatus: (fields) => {
      return "FIXME: request status"; // FIXME: // CBOR
    },
  })(request);
  // TODO: decode body from CBOR
  return config.runFetch(Endpoint.read, body);
};

const call = (
  config: Config,
) => async ({
  methodName,
  arg,
}: {
  methodName: string,
  arg: Blob,
}): Promise<SubmitResponse> => {
  const request = submitCall({
    canisterId: config.canisterId,
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
  requestId: number,
}): Promise<Response> => {
  const request = readRequestStatus({ requestId });
  return read(config)(request);
};


const API_VERSION = "v1";

interface Options {
  canisterId: number;
  fetch?: WindowOrWorkerGlobalScope["fetch"];
  host?: string;
}

interface DefaultOptions {
  fetch: WindowOrWorkerGlobalScope["fetch"];
  host: string;
}

const defaultOptions: DefaultOptions = {
  fetch: window.fetch,
  host: "http://localhost:8080",
};


interface Config {
  canisterId: number;
  host: string;
  runFetch(endpoint: Endpoint, body?: BodyInit | null): Promise<Response>;
}

const makeConfig = (options: Options): Config => {
  const withDefaults = { ...defaultOptions, ...options };
  return {
    ...withDefaults,
    runFetch: (endpoint, body) => {
      return withDefaults.fetch(`${withDefaults.host}/api/${API_VERSION}/${Endpoint[endpoint]}`, {
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
  read,
  submit,
}

export interface ApiClient {
  call(_: {
    methodName: string,
    arg: Blob,
  }): Promise<SubmitResponse>;

  requestStatus(_: {
    requestId: number,
  }): Promise<Response>;
}

export const makeApiClient = (options: Options): ApiClient => {
  const config = makeConfig(options);
  return {
    call: call(config),
    requestStatus: requestStatus(config),
  };
};
