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

interface Config {
  canisterId: number;
  host: string;
  runFetch(endpoint: Endpoint, body?: BodyInit | null): Promise<Response>;
}

enum Endpoint {
  read,
  submit,
}

type ReadRequest
  = ReadQuery
  | ReadRequestStatus;

enum ReadRequestType {
  query,
  requestStatus,
}

interface ReadQuery {
  type: ReadRequestType.query;
  canister_id: number;
  method_name: string;
  arg: Blob;
}

interface ReadRequestStatus {
  type: ReadRequestType.requestStatus;
  request_id: number;
}

const readQuery = ({
  canisterId,
  methodName,
  arg,
}: {
  canisterId: number,
  methodName: string,
  arg: Blob,
}): ReadQuery => ({
  type: ReadRequestType.query,
  canister_id: canisterId,
  method_name: methodName,
  arg,
});

const readRequestStatus = ({
  requestId,
}: {
  requestId: number,
}): ReadRequestStatus => ({
  type: ReadRequestType.requestStatus,
  request_id: requestId,
});

export enum ReadRequestStatusResponseStatus {
  unknown,
  pending,
  replied,
  rejected,
}

type SubmitRequest
  = SubmitCall;

enum SubmitRequestType {
  call,
}

interface SubmitCall {
  type: SubmitRequestType.call;
  canister_id: number;
  method_name: string;
  arg: Blob;
}

const submitCall = ({
  canisterId,
  methodName,
  arg,
}: {
  canisterId: number,
  methodName: string,
  arg: Blob,
}) => ({
  type: SubmitRequestType.call,
  canister_id: canisterId,
  method_name: methodName,
  arg,
});

const defaultOptions: DefaultOptions = {
  fetch: window.fetch,
  host: "http://localhost:8080",
};

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

interface SubmitResponse {
  requestId: number;
  response: Response;
}

const submit = (
  config: Config,
) => async (
  request: SubmitRequest,
): Promise<SubmitResponse> => {
  const body = (() => {
    switch (request.type) {
      case SubmitRequestType.call: {
        const fields = {
          request_type: request.type,
          canister_id: request.canister_id,
          method_name: request.method_name,
          arg: request.arg,
          // expiry,
          // nonce, // FIXME: provide this to create distinct request IDs
          // sender,
          // sender_pubkey,
          // sender_sig,
        };
        // FIXME: convert `fields` to `body`
        return "FIXME: call";
      }
    }
  })();
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
  const body = (() => {
    switch (request.type) {
      case ReadRequestType.query: {
        const fields = {
          request_type: request.type,
          canister_id: request.canister_id,
          method_name: request.method_name,
          arg: request.arg,
        };
        return "FIXME: query";
      }
      case ReadRequestType.requestStatus: {
        const fields = {
          request_type: request.type,
          request_id: request.request_id,
        };
        return "FIXME: request status";
      }
    }
  })();
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

export interface ApiClient {
  call({
    methodName,
    arg,
  }: {
    methodName: string,
    arg: Blob,
  }): Promise<SubmitResponse>;
  requestStatus({
    requestId,
  }: {
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
