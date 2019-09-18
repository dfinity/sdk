declare class Message {}
declare class Type {}
declare class Text {}

declare module IDL {
  class ActorInterface {
    __fields: object
    constructor(fields: object)
  }

  function idlHash(s: string): number

  function Message(argTypes?: Array<Type>, retTypes?: Array<Type>): Message
  function Text(): Text
}

export = IDL;
