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
