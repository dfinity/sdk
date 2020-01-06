import { Buffer } from 'buffer/';
import { createKeyPairFromSeed, makeAuthTransform, SenderSig, sign, verify } from './auth';
import { CanisterId } from './canisterId';
import * as cbor from './cbor';
import { HttpAgent } from './http_agent';
import { makeNonceTransform } from './http_agent_transforms';
import {
  CallRequest,
  ReadRequestType,
  RequestStatusResponseReplied,
  RequestStatusResponseStatus,
  SubmitRequestType,
} from './http_agent_types';
import { requestIdOf } from './request_id';
import { BinaryBlob } from './types';
import { Nonce } from './types';

test('call', async () => {
  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    return Promise.resolve(
      new Response(null, {
        status: 200,
      }),
    );
  });

  const canisterId: CanisterId = CanisterId.fromHex('0000000000000001');
  const nonce = Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce;
  // prettier-ignore
  const seed = Buffer.from([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
  ]);
  const keyPair = createKeyPairFromSeed(seed);

  const httpAgent = new HttpAgent({
    fetch: mockFetch,
  });
  httpAgent.addTransform(makeNonceTransform(() => nonce));
  httpAgent.addTransform(makeAuthTransform(keyPair));

  const methodName = 'greet';
  const arg = Buffer.from([]) as BinaryBlob;

  const { requestId } = await httpAgent.call(canisterId, {
    methodName,
    arg,
  });

  const mockPartialRequest = {
    request_type: SubmitRequestType.Call,
    canister_id: canisterId,
    method_name: methodName,
    // We need a request id for the signature and at the same time we
    // are checking that signature does not impact the request id.
    arg,
    nonce,
  };

  const mockPartialsRequestId = await requestIdOf(mockPartialRequest);
  const senderSig = sign(keyPair.secretKey)(mockPartialsRequestId);
  // Just sanity checking our life.
  expect(verify(mockPartialsRequestId, senderSig, keyPair.publicKey)).toBe(true);

  const expectedRequest: CallRequest = {
    ...mockPartialRequest,
    sender_pubkey: keyPair.publicKey,
    sender_sig: senderSig,
  };

  const expectedRequestId = await requestIdOf(expectedRequest);
  expect(expectedRequestId).toEqual(mockPartialsRequestId);

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(requestId).toEqual(expectedRequestId);

  expect(calls[0][0]).toBe('/api/v1/submit');
  expect(calls[0][1]).toEqual({
    method: 'POST',
    headers: {
      'Content-Type': 'application/cbor',
    },
    body: cbor.encode(expectedRequest),
  });
});

test.todo('query');

test('requestStatus', async () => {
  const mockResponse = {
    status: 'replied',
    reply: { arg: Buffer.from([]) as BinaryBlob },
  };

  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    const body = cbor.encode(mockResponse);
    return Promise.resolve(
      new Response(body, {
        status: 200,
      }),
    );
  });

  const canisterIdent = '0000000000000001';
  const nonce = Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce;

  // prettier-ignore
  const seed = Buffer.from([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
  ]);
  const keyPair = createKeyPairFromSeed(seed);
  const senderPubKey = keyPair.publicKey;

  const httpAgent = new HttpAgent({
    fetch: mockFetch,
  });
  httpAgent.addTransform(makeNonceTransform(() => nonce));
  httpAgent.addTransform(makeAuthTransform(keyPair, () => () => Buffer.from([0]) as SenderSig));

  const requestId = await requestIdOf({
    request_type: SubmitRequestType.Call,
    nonce,
    canister_id: CanisterId.fromHex(canisterIdent),
    method_name: 'greet',
    arg: Buffer.from([]),
  });

  const response = await httpAgent.requestStatus({
    requestId,
  });

  const expectedRequest = {
    request_type: ReadRequestType.RequestStatus,
    request_id: requestId,
    nonce,
    sender_pubkey: senderPubKey,
    sender_sig: Buffer.from([0]) as SenderSig,
  };

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);

  // Trick the type system.
  const {
    reply: { arg: responseArg },
    ...responseRest
  } = response as RequestStatusResponseReplied;

  const {
    reply: { arg: mockResponseArg },
    ...mockResponseRest
  } = mockResponse;

  expect(responseRest).toEqual(mockResponseRest);
  expect(responseArg.equals(mockResponseArg)).toBe(true);

  expect(calls[0]).toEqual([
    '/api/v1/read',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/cbor',
      },
      body: cbor.encode(expectedRequest),
    },
  ]);
});
