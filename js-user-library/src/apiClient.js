const API_VERSION = "v1";

const defaultConfig = {
  canisterId: null,
  fetch: window.fetch,
  host: "http://localhost:8080",
};

const makeConfig = (configArg) => {
  const config = { ...defaultConfig, ...configArg };
  return {
    ...config,
    fetch: (endpoint, body) => {
      return config.fetch(`${config.host}/api/${API_VERSION}/${endpoint}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/cbor",
        },
        body,
      });
    },
  };
};

const submit = (config) => async (requestType, fieldsArg) => {
  const fields = {
    ...fieldsArg,
    request_type: requestType,
    // expiry,
    // nonce,
    // sender,
    // sender_pubkey,
    // sender_sig,
  };
  // FIXME: convert `fields` to `body`
  const body = new Blob([], { type: "application/cbor" });
  return config.fetch("submit", body);
};

const call = (config) => async ({ methodName, arg }) => {
  return submit(config)("call", {
    canister_id: config.canisterId,
    method_name: methodName,
    arg,
  });
};

export const apiClient = (configArg) => {
  const config = makeConfig(configArg);
  return {
    call: call(config),
  };
};
