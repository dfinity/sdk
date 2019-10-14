import { sign } from "tweetnacl";
import { BinaryBlob } from "./blob";
import * as canisterId from "./canisterId";
import * as cbor from "./cbor";
import { Hex } from "./hex";
import { makeHttpAgent } from "./index";
import { Nonce } from "./nonce";
import { Request } from "./request";
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
  const keyPair = sign.keyPair.fromSeed(seed);
  const senderSecretKey = keyPair.secretKey as SenderSecretKey;

  // XXX : This should cause the test to fail.
  const senderSig = Uint8Array.from(
    [ 96,
     107,
     76,
     157,
     245,
     193,
     250,
     78,
     39,
     25,
     103,
     233,
     27,
     166,
     49,
     23,
     217,
     123,
     193,
      64,
      51,
      114,
      110,
      68,
      45,
      233,
      198,
      188,
      17,
      173,
      163,
      99,
      50,
      185,
      230,
      156,
      47,
      101,
      47,
      180,
      55,
      204,
      72,
      49,
      64,
      23,
      175,
      57,
      24,
      174,
      174,
      109,
      211,
      175,
      28,
      93,
      253,
      231,
      127,
      99,
      184,
      188,
      143,
      64,
      11,
      192,
      172,
      131,
      201,
      239,
      28,
      191,
      191,
      184,
      43,
      71,
      195,
      19,
      218,
      182,
      227,
      149,
      107,
      40,
      42,
      223,
      248,
      0,
      179,
      154,
      140,
      251,
      162,
      171,
      241,
      64,
      222]) as SenderSig;

  const httpAgent = makeHttpAgent({
    canisterId: canisterIdent,
    fetchFn: mockFetch,
    nonceFn: () => nonce,
    senderSecretKey,
//    senderSigFn: () => senderSig,
  });

  const methodName = "greet";
  const arg = Uint8Array.from([]) as BinaryBlob;

  const { requestId, response } = await httpAgent.call({
    methodName,
    arg,
  });

  const expectedRequest: Request = {
    request_type: "call" as RequestType,
    nonce,
    canister_id: canisterId.fromHex(canisterIdent),
    method_name: methodName,
    arg,
    sender_pubkey: keyPair.publicKey as SenderPubKey,
    sender_sig: senderSig,
  };

  const expectedRequestId = await requestIdOf(expectedRequest);

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
  const keyPair = sign.keyPair.fromSeed(seed);
  const senderSecretKey = keyPair.secretKey as SenderSecretKey;

  // XXX : This should cause the test to fail!!!
  const senderSig = Uint8Array.from([
    96,
    135,
    218,
    78,
    43,
    194,
    20,
    65,
    207,
    1,
    17,
    220,
    21,
    55,
    73,
    84,
    26,
    127,
    216,
    167,
    18,
    1,
    190,
    0,
    142,
    134,
    240,
    7,
    19,
    197,
    17,
    195,
    254,
    20,
    172,
    40,
    88,
    213,
    119,
    254,
    199,
    28,
    205,
    78,
    198,
    117,
    205,
    92,
    104,
    252,
    54,
    94,
    224,
    114,
    95,
    164,
    205,
    110,
    67,
    176,
    184,
    161,
    64,
    161,
    11,
    92,
    67,
    61,
    182,
    100,
    228,
    105,
    79,
    55,
    215,
    85,
    143,
    143,
    94,
    119,
    239,
    168,
    1,
    46,
    146,
    194,
    183,
    133,
    141,
    138,
    168,
    87,
    228,
    64,
    33,
    248,
    50]) as SenderSig;

  const httpAgent = makeHttpAgent({
    canisterId: canisterIdent,
    fetchFn: mockFetch,
    nonceFn: () => nonce,
    senderSecretKey,
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
    sender_pubkey: keyPair.publicKey as SenderPubKey,
    sender_sig: senderSig,
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
