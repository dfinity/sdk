import { apiClient, IDL, makeActor } from "./index";

test("call", async () => {
  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Message([IDL.Text], [IDL.Text]),
  });
  const responseValue = "Hello, World!";
  const mockFetch = jest.fn((resource, init) => Promise.resolve(responseValue));
  const client = apiClient({
    canisterId: 1,
    fetch: mockFetch,
  });
  const actor = makeActor(actorInterface)(client);
  const response = await actor.greet();
  expect(response).toBe(responseValue);

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
