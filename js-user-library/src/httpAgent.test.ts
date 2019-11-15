import { createKeyPairFromSeed, sign, verify } from "./auth";
import { BinaryBlob } from "./blob";
import * as canisterId from "./canisterId";
import * as cbor from "./cbor";
import { Hex } from "./hex";
import { makeHttpAgent } from "./index";
import { Nonce } from "./nonce";
import { CommonFields, Request } from "./request";
import { requestIdOf } from "./requestId";
import { RequestType } from "./requestType";
import { SenderPubKey } from "./senderPubKey";
import { SenderSecretKey } from "./senderSecretKey";
import { SenderSig } from "./senderSig";


test("call", async () => {
  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    return Promise.resolve(new Response(null, {
      status: 200,
    }));
  });

  const canisterIdent = "0000000000000001" as Hex;
  const nonce = Uint8Array.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce;
  const seed = Uint8Array.from(
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
  const keyPair = createKeyPairFromSeed(seed);
  const senderPubKey = keyPair.publicKey as SenderPubKey;
  const senderSecretKey = keyPair.secretKey as SenderSecretKey;

  const httpAgent = makeHttpAgent({
    canisterId: canisterIdent,
    fetchFn: mockFetch,
    nonceFn: () => nonce,
    senderSecretKey,
    senderPubKey,
  });

  const methodName = "greet";
  const arg = Uint8Array.from([]) as BinaryBlob;

  const { requestId, response } = await httpAgent.call({
    methodName,
    arg,
  });

  const mockPartialRequest: CommonFields = {
    request_type: "call" as RequestType,
    nonce,
    canister_id: canisterId.fromHex(canisterIdent),
    method_name: methodName,
    // We need a request id for the signature and at the same time we
    // are checking that signature does not impact the request id.
    arg,
  };

  const mockPartialsRequestId = await requestIdOf(mockPartialRequest);
  const senderSig =
    sign(senderSecretKey)(mockPartialsRequestId);
  // Just sanity checking our life.
  expect(
    verify(mockPartialsRequestId, senderSig, senderPubKey))
    .toBe(true);

  const expectedRequest: Request = {
    ...mockPartialRequest,
    sender_pubkey: keyPair.publicKey as SenderPubKey,
    sender_sig: senderSig,
  };

  const expectedRequestId = await requestIdOf(expectedRequest);
  expect(expectedRequestId).toEqual(mockPartialsRequestId);

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(requestId).toEqual(expectedRequestId);

  expect(calls[0][0]).toBe("http://localhost:8000/api/v1/submit");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode(expectedRequest),
  });
});

test.todo("query");

test("requestStatus", async () => {

  const mockResponse = {
    status: "replied",
    reply: { arg: Uint8Array.from([]) as BinaryBlob },
  };

  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    const body = cbor.encode(mockResponse);
    return Promise.resolve(new Response(body, {
      status: 200,
    }));
  });

  const canisterIdent = "0000000000000001" as Hex;
  const nonce = Uint8Array.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce;

  const seed = Uint8Array.from(
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
  const keyPair = createKeyPairFromSeed(seed);
  const senderSecretKey = keyPair.secretKey as SenderSecretKey;
  const senderPubKey = keyPair.publicKey as SenderPubKey;


  const httpAgent = makeHttpAgent({
    canisterId: canisterIdent,
    fetchFn: mockFetch,
    nonceFn: () => nonce,
    senderSecretKey,
    senderPubKey,
    senderSigFn: (x) => (req) =>
      Uint8Array.from([0])  as SenderSig,
  });

  const requestId = await requestIdOf({
    request_type: "call" as RequestType,
    nonce,
    canister_id: canisterId.fromHex(canisterIdent),
    method_name: "greet",
    arg: Uint8Array.from([]),
  });

  const response = await httpAgent.requestStatus({
    requestId,
  });

  const expectedRequest: Request = {
    request_type: "request-status" as RequestType,
    nonce,
    request_id: requestId,
    sender_pubkey: senderPubKey,
    sender_sig: Uint8Array.from([0]) as SenderSig,
  };

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);

  const {
    reply: { arg: responseArg },
    ...responseRest
  } = response;

  const {
    reply: { arg: mockResponseArg },
    ...mockResponseRest
  } = mockResponse;

  expect(responseRest).toEqual(mockResponseRest);
  expect(responseArg.equals(mockResponseArg)).toBe(true);

  expect(calls[0][0]).toBe("http://localhost:8000/api/v1/read");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode(expectedRequest),
  });
});
