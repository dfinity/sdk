import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import * as canisterId from "./canisterId";
import * as cbor from "./cbor";
import { Hex } from "./hex";
import { Nonce } from "./nonce";
import { Request } from "./request";
import { requestIdOf } from "./requestId";
import { SenderPubKey } from "./senderPubKey";
import { SenderSecretKey } from "./senderSecretKey";
import { SenderSig } from "./senderSig";

import {
  IDL as _IDL,
  makeActor,
  makeHttpAgent,
} from "./index";

test("makeActor", async () => {
  const actorInterface = ({ IDL }: { IDL: typeof _IDL }) => {
    return new IDL.ActorInterface({
      greet: IDL.Func([IDL.Text], [IDL.Text]),
    });
  };

  const expectedReplyArg = new Uint8Array(
    _IDL.Text.encode("Hello, World!").buffer,
  ) as BinaryBlob;

  const mockFetch: jest.Mock = jest.fn()
    .mockImplementationOnce((/*resource, init*/) => {
      return Promise.resolve(new Response(null, {
        status: 202,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: "unknown" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: "pending" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({
        status: "replied",
        reply: {
          arg: expectedReplyArg,
        },
      });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    });

  const methodName = "greet";
  const arg = Uint8Array.from([]);

  const canisterIdent = "0000000000000001" as Hex;
  const senderPubKey = new Uint8Array(32) as SenderPubKey;
  const senderSecretKey = new Uint8Array(32) as SenderSecretKey;
  const senderSig = Uint8Array.from([0]) as SenderSig;

  const nonces = [
    Uint8Array.from([0, 1, 2, 3, 4, 5, 6, 7]) as Nonce,
    Uint8Array.from([1, 2, 3, 4, 5, 6, 7, 8]) as Nonce,
    Uint8Array.from([2, 3, 4, 5, 6, 7, 8, 9]) as Nonce,
    Uint8Array.from([3, 4, 5, 6, 7, 8, 9, 0]) as Nonce,
  ];

  const expectedCallRequest = {
    request_type: "call",
    nonce: nonces[0],
    canister_id: canisterId.fromHex(canisterIdent),
    method_name: methodName,
    arg,
    sender_pubkey: senderPubKey,
    sender_sig: senderSig,
  } as Request;

  const expectedCallRequestId = await requestIdOf(expectedCallRequest);

  let nonceCount = 0;

  const httpAgent = makeHttpAgent({
    canisterId: canisterIdent,
    fetchFn: mockFetch,
    nonceFn: () => {
      const nonce = nonces[nonceCount];
      nonceCount = nonceCount + 1;
      return nonce;
    },
    senderSecretKey,
    senderPubKey,
    senderSigFn: (x) => (req) =>
      Uint8Array.from([0])  as SenderSig,
  });

  const actor = makeActor(actorInterface)(httpAgent);
  // FIXME: the argument isn't actually used yet
  const reply = await actor.greet("Name");

  expect(
    blob.toHex(reply),
  ).toEqual(
    blob.toHex(expectedReplyArg),
  );

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(4);

  expect(calls[0][0]).toBe("http://localhost:8000/api/v1/submit");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode(expectedCallRequest),
  });

  expect(calls[1][0]).toBe("http://localhost:8000/api/v1/read");
  expect(calls[1][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      nonce: nonces[1],
      request_id: expectedCallRequestId,
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });

  expect(calls[2][0]).toBe("http://localhost:8000/api/v1/read");
  expect(calls[2][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      nonce: nonces[2],
      request_id: expectedCallRequestId,
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });

  expect(calls[3][0]).toBe("http://localhost:8000/api/v1/read");
  expect(calls[3][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      nonce: nonces[3],
      request_id: expectedCallRequestId,
      sender_pubkey: senderPubKey,
      sender_sig: senderSig,
    }),
  });
});

// TODO: tests for rejected, unknown time out
