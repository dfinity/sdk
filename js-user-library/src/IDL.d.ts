declare class _Message {}
declare class _Type {}
declare class _Text {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Message(argTypes?: Array<_Type>, retTypes?: Array<_Type>): _Message
export function Text(): _Text
