// Relevant links:
// * New binary format spec
//   https://github.com/dfinity-lab/actorscript/blob/b240d8d28a6cef470faa56cf7322127819111fbc/design/IDL.md#binary-format
// * Old implementation based on a temporary wire format for ActorScript types
//   https://github.com/dfinity-lab/dev/blob/9030c90efe5b3de33670d4f4f0331482d51c5858/experimental/js-dfinity-client/src/IDL.js#L213


export const hash = (s: string): number => {
  const utf8encoder = new TextEncoder();
  const array = utf8encoder.encode(s);
  let h = 0;
  array.forEach((c, i, array) => { h *= 223; h += c; h %= 2**32; });
  return h;
}
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
