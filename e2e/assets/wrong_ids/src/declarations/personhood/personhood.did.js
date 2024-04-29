export const idlFactory = ({ IDL }) => {
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponsePayload = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponsePayload,
  });
  return IDL.Service({
    'getEthereumSigningMessage' : IDL.Func(
        [],
        [IDL.Record({ 'message' : IDL.Text, 'nonce' : IDL.Text })],
        [],
      ),
    'removeHTTPHeaders' : IDL.Func(
        [TransformArgs],
        [HttpResponsePayload],
        ['query'],
      ),
    'scoreBySignedEthereumAddress' : IDL.Func(
        [
          IDL.Record({
            'signature' : IDL.Text,
            'address' : IDL.Text,
            'nonce' : IDL.Text,
          }),
        ],
        [IDL.Text],
        [],
      ),
    'submitSignedEthereumAddressForScore' : IDL.Func(
        [
          IDL.Record({
            'signature' : IDL.Text,
            'address' : IDL.Text,
            'nonce' : IDL.Text,
          }),
        ],
        [IDL.Text],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
