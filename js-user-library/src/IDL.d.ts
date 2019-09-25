export interface Func {
  argTypes: Array<Type>;
  retTypes: Array<Type>;
}

interface JsArray extends Array<JsValue> {}
type JsValue = boolean | string | number | JsArray | object

export interface Type {
  // encode(x: JsValue): Buffer // A safe-buffer Buffer
  // decode(x: Buffer): JsValue // A safe-buffer Buffer
}

export interface Text extends Type {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Func(argTypes?: Array<Type>, retTypes?: Array<Type>): Func
export const Text: Text
