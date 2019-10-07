import { Buffer } from "buffer";

export interface Func {
  argTypes: Array<Type>;
  retTypes: Array<Type>;
}

interface JsArray extends Array<JsValue> {}
type JsValue = boolean | string | number | JsArray | object

export interface Type {
  // NOTE: these are actually buffers from the `safe-buffer` package, but we type
  // them as being from the `buffer` package for compatibility.
  encode(x: JsValue): Buffer
  decode(x: Buffer): JsValue
}

// export interface Obj extends Type {}
export interface Text extends Type {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Func(argTypes?: Array<Type>, retTypes?: Array<Type>): Func
// export function Obj(fields: Record<string, Type>): Obj
export const Text: Text
