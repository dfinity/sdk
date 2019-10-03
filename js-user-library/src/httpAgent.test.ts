import BigNumber from "bignumber.js";
import * as cbor from "./cbor";

import {
  CanisterId,
  Int,
  makeHttpAgent,
  Request,
  requestIdOf,
} from "./index";

test("call", async () => {
  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    return Promise.resolve(new Response(null, {
      status: 200,
    }));
  });

  const canisterId = new BigNumber(1);

  const httpAgent = makeHttpAgent({
    canisterId,
    fetch: mockFetch,
  });

  const methodName = "greet";
  const arg: Array<Int> = [];

  const { requestId, response } = await httpAgent.call({
    methodName,
    arg,
  });

  const expectedRequest = {
    request_type: "call",
    canister_id: canisterId,
    method_name: methodName,
    arg,
  } as Request;

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
    reply: { arg: [] },
  };

  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    const body = cbor.encode(mockResponse);
    return Promise.resolve(new Response(body, {
      status: 200,
    }));
  });

  const canisterId = new BigNumber(1);

  const httpAgent = makeHttpAgent({
    canisterId,
    fetch: mockFetch,
  });

  const requestId = await requestIdOf({
    request_type: "call",
    canister_id: canisterId,
    method_name: "greet",
    arg: [],
  } as Request);

  const response = await httpAgent.requestStatus({
    requestId,
  });

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(response).toEqual(mockResponse);

  expect(calls[0][0]).toBe("http://localhost:8000/api/v1/read");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      request_id: requestId,
    }),
  });
});
