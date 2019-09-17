export class ActorInterface {
  fields: object;

  constructor(fields: object) {
    this.fields = fields;
  }
};


class _Message {
  argTypes: Array<Type>
  returnTypes: Array<Type>;

  constructor(argTypes: Array<Type>, returnTypes: Array<Type>) {
    this.argTypes = argTypes;
    this.returnTypes = returnTypes;
  }
};

export const Message = (argTypes: Array<Type>, returnTypes: Array<Type>) => {
  return new _Message(argTypes, returnTypes);
};


// TODO: try using built-in ArrayBuffer before reaching for external packages
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer

class Type {};


class _Text extends Type {};

export const Text = () => new _Text();
