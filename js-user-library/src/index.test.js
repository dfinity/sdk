import { IDL, makeActor, sum } from "./index";

test("Hello, World!", async () => {
  class TestApiClient {
    async call() {
      return "Hello, World!";
    }
  }

  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Message([IDL.Text], [IDL.Text]),
  });
  const client = new TestApiClient();
  const actor = makeActor(actorInterface)(client);
  const response = await actor.greet();
  expect(response).toBe("Hello, World!");
});

// TODO: test ApiClient
