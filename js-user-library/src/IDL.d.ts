export interface Function {}
export interface Type {}
export interface Text {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Function(argTypes?: Array<Type>, retTypes?: Array<Type>): Function
export function Text(): Text
