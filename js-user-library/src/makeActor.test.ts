import { IDL, makeActor } from "./index";

test("makeActor", async () => {
  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Message([IDL.Text], [IDL.Text]),
  });
  const responseValue = "Hello, World!";
  const apiClient = {
    call: () => responseValue,
  };
  const actor = makeActor(actorInterface)(apiClient);
  const response = await actor.greet();
  expect(response).toBe(responseValue);
});
