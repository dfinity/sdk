import { IDL, makeActor, makeApiClient } from "./index";

test("call", async () => {
  const greeting = "Hello, World!";

  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    // FIXME: the body should be a CBOR value
    // status: "replied", reply: greeting
    return Promise.resolve(new Response(greeting, {
      status: 200,
    }));
  });

  const apiClient = makeApiClient({
    canisterId: 1,
    fetch: mockFetch,
  });

  const { requestId, response } = await apiClient.call({
    methodName: "greet",
    arg: new Blob([], { type: "application/cbor" }), // FIXME
  });

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(requestId).toBe(-1); // FIXME
  expect(await response.text()).toBe(greeting); // FIXME
});
