import { makeApiClient, IDL, makeActor } from "./index";

test("call", async () => {
  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Message([IDL.Text], [IDL.Text]),
  });
  // FIXME: since we're making a submit call, there won't be a response
  const greeting = "Hello, World!";
  const mockFetch: jest.Mock = jest.fn((resource, init) => {
    return Promise.resolve(new Response(greeting))
  });
  const apiClient = makeApiClient({
    canisterId: 1,
    fetch: mockFetch,
  });
  const actor = makeActor(actorInterface)(apiClient);
  const response = await actor.greet(); // TODO: map
  const responseText = await response.text();
  expect(responseText).toBe(greeting);

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(1);
  expect(calls[0][0]).toBe("http://localhost:8080/api/v1/submit");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: new Blob([], { type: "application/cbor" }), // FIXME
  });
});
