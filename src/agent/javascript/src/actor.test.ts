import { Buffer } from 'buffer/';
import { makeActorFactory } from './actor';
import { makeAuthTransform, SenderPubKey, SenderSecretKey, SenderSig } from './auth';
import { CanisterId } from './canisterId';
import * as cbor from './cbor';
import { HttpAgent } from './http_agent';
import { makeNonceTransform } from './http_agent_transforms';
import {
  CallRequest,
  Signed,
  SignedHttpAgentSubmitRequest,
  SubmitRequest,
  SubmitRequestType,
} from './http_agent_types';
import * as IDL from './idl';
import { Principal } from './principal';
import { requestIdOf } from './request_id';
import { blobFromHex, Nonce } from './types';
import { sha256 } from './utils/sha256';

test('makeActor', async () => {
  const actorInterface = () => {
    return IDL.Service({
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
      const body = cbor.encode({ status: 'received' });
      return Promise.resolve(
        new Response(body, {
          status: 200,
        }),
      );
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: 'processing' });
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
  const principal = await Principal.selfAuthenticating(senderPubKey);
  const sender = principal.toBlob();

  const nonces = [
    Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce,
    Buffer.from([1, 2, 3, 4, 5, 6, 7, 8]) as Nonce,
    Buffer.from([2, 3, 4, 5, 6, 7, 8, 9]) as Nonce,
    Buffer.from([3, 4, 5, 6, 7, 8, 9, 0]) as Nonce,
    Buffer.from([4, 5, 6, 7, 8, 9, 0, 1]) as Nonce,
  ];

  const expectedCallRequest = {
    content: {
      request_type: SubmitRequestType.Call,
      canister_id: canisterId,
      method_name: methodName,
      arg,
      nonce: nonces[0],
      sender,
    },
    sender_pubkey: senderPubKey,
    sender_sig: senderSig,
  } as Signed<CallRequest>;

  const expectedCallRequestId = await requestIdOf(expectedCallRequest.content);

  let nonceCount = 0;

  const httpAgent = new HttpAgent({
    fetch: mockFetch,
    principal,
  });
  httpAgent.addTransform(makeNonceTransform(() => nonces[nonceCount++]));
  httpAgent.setAuthTransform(
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

  expect(calls.length).toBe(5);
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
        content: {
          request_type: 'request_status',
          request_id: expectedCallRequestId,
          nonce: nonces[1],
          sender,
        },
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
      content: {
        request_type: 'request_status',
        request_id: expectedCallRequestId,
        nonce: nonces[2],
        sender,
      },
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
      content: {
        request_type: 'request_status',
        request_id: expectedCallRequestId,
        nonce: nonces[3],
        sender,
      },
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });

  expect(calls[4][0]).toBe('/api/v1/read');
  expect(calls[4][1]).toEqual({
    method: 'POST',
    headers: {
      'Content-Type': 'application/cbor',
    },
    body: cbor.encode({
      content: {
        request_type: 'request_status',
        request_id: expectedCallRequestId,
        nonce: nonces[4],
      },
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });
});

// TODO: tests for rejected, unknown time out
