declare class _Function {}
declare class _Type {}
declare class _Text {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Function(argTypes?: Array<_Type>, retTypes?: Array<_Type>): _Function
export function Text(): _Text
