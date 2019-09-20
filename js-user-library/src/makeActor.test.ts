import { CanisterId, IDL, makeActor, makeApiClient } from "./index";

test("makeActor", async () => {
  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Fn([IDL.Text], [IDL.Text]),
  });

  const name = "World";
  const expectedReply = `Hello, ${name}!`;

  const mockFetch: jest.Mock = jest.fn()
    .mockImplementationOnce((resource, init) => {
      return Promise.resolve(new Response(null, {
        status: 202,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      // FIXME: the body should be a CBOR value
      const body = JSON.stringify({ status: "unknown" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      // FIXME: the body should be a CBOR value
      const body = JSON.stringify({ status: "pending" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      // FIXME: the body should be a CBOR value
      const body = JSON.stringify({ status: "replied", reply: expectedReply });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    });

  const apiClient = makeApiClient({
    canisterId: 1 as CanisterId,
    fetch: mockFetch,
  });

  const actor = makeActor(actorInterface)(apiClient);
  const reply = await actor.greet(name);

  expect(reply).toBe(expectedReply);

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(4);

  expect(calls[0][0]).toBe("http://localhost:8080/api/v1/submit");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    // FIXME
    // body: new Blob([], { type: "application/cbor" }),
    body: "FIXME: call", // FIXME: use name
  });

  expect(calls[1][0]).toBe("http://localhost:8080/api/v1/read");
  expect(calls[1][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    // FIXME
    // body: new Blob([], { type: "application/cbor" }),
    body: "FIXME: request status",
  });

  expect(calls[2][0]).toBe("http://localhost:8080/api/v1/read");
  expect(calls[2][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    // FIXME
    // body: new Blob([], { type: "application/cbor" }),
    body: "FIXME: request status",
  });

  expect(calls[3][0]).toBe("http://localhost:8080/api/v1/read");
  expect(calls[3][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    // FIXME
    // body: new Blob([], { type: "application/cbor" }),
    body: "FIXME: request status",
  });
});
