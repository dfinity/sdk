import { Buffer } from 'buffer/';
import { makeActorFactory } from './actor';
import { makeAuthTransform, SenderPubKey, SenderSecretKey, SenderSig } from './auth';
import { CanisterId } from './canisterId';
import * as cbor from './cbor';
import { HttpAgent } from './http_agent';
import { makeNonceTransform } from './http_agent_transforms';
import { SubmitRequestType } from './http_agent_types';
import * as IDL from './idl';
import { requestIdOf } from './request_id';
import { blobFromHex, Nonce } from './types';

test('makeActor', async () => {
  const actorInterface = () => {
    return new IDL.ActorInterface({
      greet: IDL.Func([IDL.Text], [IDL.Text]),
    });
  };

  const expectedReplyArg = blobFromHex(IDL.encode([IDL.Text], ['Hello, World!']).toString('hex'));

  const mockFetch: jest.Mock = jest
    .fn()
    .mockImplementationOnce((/*resource, init*/) => {
      return Promise.resolve(
        new Response(null, {
          status: 202,
        }),
      );
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: 'unknown' });
      return Promise.resolve(
        new Response(body, {
          status: 200,
        }),
      );
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: 'pending' });
      return Promise.resolve(
        new Response(body, {
          status: 200,
        }),
      );
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({
        status: 'replied',
        reply: {
          arg: expectedReplyArg,
        },
      });
      return Promise.resolve(
        new Response(body, {
          status: 200,
        }),
      );
    });

  const methodName = 'greet';
  const argValue = 'Name';

  const arg = blobFromHex(IDL.encode([IDL.Text], [argValue]).toString('hex'));

  const canisterId: CanisterId = CanisterId.fromText('ic:000000000000000107');
  const senderPubKey = Buffer.alloc(32, 0) as SenderPubKey;
  const senderSecretKey = Buffer.alloc(32, 0) as SenderSecretKey;
  const senderSig = Buffer.from([0]) as SenderSig;

  const nonces = [
    Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce,
    Buffer.from([1, 2, 3, 4, 5, 6, 7, 8]) as Nonce,
    Buffer.from([2, 3, 4, 5, 6, 7, 8, 9]) as Nonce,
    Buffer.from([3, 4, 5, 6, 7, 8, 9, 0]) as Nonce,
  ];

  const expectedCallRequest = {
    request_type: SubmitRequestType.Call,
    canister_id: canisterId,
    method_name: methodName,
    arg,
    nonce: nonces[0],
    sender_pubkey: senderPubKey,
    sender_sig: senderSig,
  };

  const expectedCallRequestId = await requestIdOf(expectedCallRequest);

  let nonceCount = 0;

  const httpAgent = new HttpAgent({
    fetch: mockFetch,
  });
  httpAgent.addTransform(makeNonceTransform(() => nonces[nonceCount++]));
  httpAgent.addTransform(
    makeAuthTransform(
      {
        publicKey: senderPubKey,
        secretKey: senderSecretKey,
      },
      () => () => Buffer.from([0]) as SenderSig,
    ),
  );

  const actor = makeActorFactory(actorInterface)({ canisterId, httpAgent });
  const reply = await actor.greet(argValue);

  expect(reply).toEqual(IDL.decode([IDL.Text], expectedReplyArg)[0]);

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(4);
  expect(calls[0]).toEqual([
    '/api/v1/submit',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/cbor',
      },
      body: cbor.encode(expectedCallRequest),
    },
  ]);

  expect(calls[1]).toEqual([
    '/api/v1/read',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/cbor',
      },
      body: cbor.encode({
        request_type: 'request_status',
        request_id: expectedCallRequestId,
        nonce: nonces[1],
        sender_pubkey: senderPubKey,
        sender_sig: senderSig,
      }),
    },
  ]);

  expect(calls[2][0]).toBe('/api/v1/read');
  expect(calls[2][1]).toEqual({
    method: 'POST',
    headers: {
      'Content-Type': 'application/cbor',
    },
    body: cbor.encode({
      request_type: 'request_status',
      request_id: expectedCallRequestId,
      nonce: nonces[2],
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });

  expect(calls[3][0]).toBe('/api/v1/read');
  expect(calls[3][1]).toEqual({
    method: 'POST',
    headers: {
      'Content-Type': 'application/cbor',
    },
    body: cbor.encode({
      request_type: 'request_status',
      request_id: expectedCallRequestId,
      nonce: nonces[3],
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });
});

// TODO: tests for rejected, unknown time out
