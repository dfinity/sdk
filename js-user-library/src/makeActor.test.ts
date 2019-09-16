import { IDL, makeActor } from "./index";

test("makeActor", async () => {
  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Message([IDL.Text], [IDL.Text]),
  });
  const greeting = "Hello, World!";
  const apiClient = {
    call: () => Promise.resolve(new Response(greeting)),
  };
  const actor = makeActor(actorInterface)(apiClient);
  const response = await actor.greet(); // TODO: map
  const responseText = await response.text();
  expect(responseText).toBe(greeting);
});
