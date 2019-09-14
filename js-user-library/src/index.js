// Allows for one client for the lifetime of the actor:
//
// ```
// const actor = makeActor(actorInterface)(client);
// const response = actor.greet();
// ```
//
// or using a different client for the same actor if necessary:
//
// ```
// const actor = makeActor(actorInterface);
// const response1 = actor(client1).greet();
// const response2 = actor(client2).greet();
// ```
export const makeActor = (actorInterface) => (client) => {
  const entries = Object.entries(actorInterface.fields);
  return Object.fromEntries(entries.map(([methodName, desc]) => {
    return [methodName, async (...args) => {
      // TODO: convert `args` to `arg` using `desc`
      const arg = new Blob([], { type: "application/cbor" });
      return client.call({ methodName, arg });
    }];
  }));
};

// TODO: it seems like these don't need to be classes and could be functions
// that return objects instead.

class ActorInterface {
  constructor(fields) {
    this.fields = fields;
  }
};

class Message {
  constructor(argTypes, returnTypes) {
    this.argTypes = argTypes;
    this.returnTypes = returnTypes;
  }
};

class Text {};

export const IDL = {
  ActorInterface,
  Message: (...args) => new Message(...args),
  Text: () => new Text(),
};
