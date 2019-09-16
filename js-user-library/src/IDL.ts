class ActorInterface {
  fields: object;

  constructor(fields: object) {
    this.fields = fields;
  }
};

class Message {
  argTypes: Array<Type>
  returnTypes: Array<Type>;

  constructor(argTypes: Array<Type>, returnTypes: Array<Type>) {
    this.argTypes = argTypes;
    this.returnTypes = returnTypes;
  }
};

class Type {};

class Text extends Type {};

export const IDL = {
  ActorInterface,
  Message: (argTypes: Array<Type>, returnTypes: Array<Type>) => {
    return new Message(argTypes, returnTypes);
  },
  Text: () => new Text(),
};
