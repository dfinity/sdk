import {
  CanisterId,
  Int,
  makeCallRequest,
  makeHttpAgent,
  requestIdOf,
} from "./index";

test("call", async () => {
  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    return Promise.resolve(new Response(null, {
      status: 200,
    }));
  });

  const canisterId = [1] as CanisterId;

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

  const expectedRequestId = await requestIdOf(
    makeCallRequest({
      canisterId,
      methodName,
      arg,
    }),
  );

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(requestId).toEqual(expectedRequestId);
});

test.todo("query");

test.todo("requestStatus");
