import { Buffer } from "buffer/";

export interface Func {
  argTypes: Array<Type>;
  retTypes: Array<Type>;
}

interface JsArray extends Array<JsValue> {}
type JsValue = boolean | string | number | JsArray | object

export interface Type {
  encodeGo(x: JsValue): Buffer
  decodeGo(x: Buffer): JsValue
}

export interface Text extends Type {}

export class ActorInterface {
  __fields: object
  constructor(fields: object)
}

export function idlHash(s: string): number

export function Func(argTypes?: Array<Type>, retTypes?: Array<Type>): Func
export const Text: Text

export function encode(argTypes: Array<Type>, args: Array<JsValue>): Buffer
export function decode(retTypes: Array<Type>, bytes: Buffer): Array<JsValue>
