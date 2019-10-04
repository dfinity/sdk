import { Buffer } from "buffer/";
import * as cbor from "./cbor";

import {
  Hex,
  IDL as _IDL,
  makeActor,
  makeHttpAgent,
  Request,
  requestIdOf,
} from "./index";

test("makeActor", async () => {
  const actorInterface = ({ IDL }: { IDL: typeof _IDL }) => {
    return new IDL.ActorInterface({
      greet: IDL.Func([IDL.Text], [IDL.Text]),
      // greet: IDL.Func(IDL.Obj({ "0": Text }), IDL.Obj({ "0": Text })),
    });
  };

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
      const body = cbor.encode({ status: "replied", reply: "Hello, World!" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    });

  const methodName = "greet";
  const arg = Buffer.from([]);

  const nonces = [
    Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]),
    Buffer.from([1, 2, 3, 4, 5, 6, 7, 8]),
    Buffer.from([2, 3, 4, 5, 6, 7, 8, 9]),
    Buffer.from([3, 4, 5, 6, 7, 8, 9, 0]),
  ];

  const expectedCallRequest = {
    request_type: "call",
    nonce: nonces[0],
    canister_id: Buffer.from([0, 0, 0, 0, 0, 0, 0, 1]),
    method_name: methodName,
    arg,
  } as Request;

  const expectedCallRequestId = await requestIdOf(expectedCallRequest);

  let nonceCount = 0;

  const httpAgent = makeHttpAgent({
    canisterId: "0000000000000001" as Hex,
    fetchFn: mockFetch,
    nonceFn: () => {
      const nonce = nonces[nonceCount];
      nonceCount = nonceCount + 1;
      return nonce;
    },
  });

  const actor = makeActor(actorInterface)(httpAgent);
  // FIXME: the argument isn't actually used yet
  const reply = await actor.greet("Name");

  expect(reply).toBe("Hello, World!");

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
    }),
  });
});

// TODO: tests for rejected, unknown time out
