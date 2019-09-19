export interface Type {}
export interface Fn {}
export interface Text {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Fn(argTypes?: Array<Type>, retTypes?: Array<Type>): Fn
export function Text(): Text
