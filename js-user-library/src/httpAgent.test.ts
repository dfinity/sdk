import { CanisterId, IDL, makeActor, makeHttpAgent } from "./index";

test("call", async () => {
  const greeting = "Hello, World!";

  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    // FIXME: the body should be a CBOR value
    // status: "replied", reply: greeting
    return Promise.resolve(new Response(greeting, {
      status: 200,
    }));
  });

  const httpAgent = makeHttpAgent({
    canisterId: [1] as CanisterId,
    fetch: mockFetch,
  });

  const { requestId, response } = await httpAgent.call({
    methodName: "greet",
    arg: [],
  });

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(requestId).toEqual([1]); // FIXME
  expect(await response.text()).toBe(greeting); // FIXME
});
