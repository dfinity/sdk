const API_VERSION = "v1";

type Options = {
  canisterId: number,
  fetch?: WindowOrWorkerGlobalScope["fetch"],
  host?: string,
};

type DefaultOptions = {
  fetch: WindowOrWorkerGlobalScope["fetch"],
  host: string,
};

type Config = {
  canisterId: number,
  runFetch(endpoint: Endpoint, body?: BodyInit | null): Promise<Response>,
  host: string,
};

enum Endpoint {
  submit,
};

enum RequestType {
  call,
};

const defaultOptions: DefaultOptions = {
  fetch: window.fetch,
  host: "http://localhost:8080",
};

const makeConfig = (options: Options): Config => {
  const withDefaults = { ...defaultOptions, ...options };
  return {
    ...withDefaults,
    runFetch: (endpoint, body) => {
      return withDefaults.fetch(`${withDefaults.host}/api/${API_VERSION}/${endpoint}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/cbor",
        },
        body,
      });
    },
  };
};

const submit = (config: Config) => async (requestType: RequestType, fields: object) => {
  const _fields = {
    ...fields,
    request_type: RequestType[requestType],
    // expiry,
    // nonce,
    // sender,
    // sender_pubkey,
    // sender_sig,
  };
  // FIXME: convert `_fields` to `body`
  const body = "FIXME";
  return config.runFetch(Endpoint.submit, body);
};

const call = (config: Config) => async ({ methodName, arg }: { methodName: string, arg: Blob }): Promise<Response> => {
  return submit(config)(RequestType.call, {
    canister_id: config.canisterId,
    method_name: methodName,
    arg,
  });
};

type ApiClient = {
  call({ methodName, arg }: { methodName: string, arg: Blob }): Promise<Response>;
}

export const makeApiClient = (options: Options): ApiClient => {
  const config = makeConfig(options);
  return {
    call: call(config),
  };
};
