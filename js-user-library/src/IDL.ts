import { Buffer } from "buffer";
import Pipe = require("buffer-pipe");
import { signed as sleb, unsigned as leb } from "leb128";

// tslint:disable:max-line-length
/**
 * This module provides a combinator library to create serializers/deserializers
 * between JavaScript values and IDL used by canisters on the Internet Computer,
 * as documented at https://github.com/dfinity-lab/actorscript/blob/128e37bf6800125056269454a21acd8f2c70b226/design/IDL.md
 */
// tslint:enable:max-line-length

function zipWith<TX, TY, TR>(
  xs: TX[],
  ys: TY[],
  f: (a: TX, b: TY) => TR,
): TR[] {
  return xs.map((x, i) => f(x, ys[i]));
}

/** @internal */
export function hash(s: string): number {
  const utf8encoder = new TextEncoder();
  const array = utf8encoder.encode(s);

  let h = 0;
  for (const c of array) {
    h = (h * 223 + c) % 2 ** 32;
  }
  return h;
}

const magicNumber = "DIDL";

export class TypeTable {
  // List of types. Needs to be an array as the index needs to be stable.
  private _typs: Buffer[] = [];
  private _idx = new Map<string, number>();

  public has(obj: string) {
    return this._idx.has(obj);
  }

  public add(obj: string, buf: Buffer) {
    if (this._idx.has(obj)) {
      throw new Error("Duplicate type name: " + obj);
    }
    const idx = this._typs.length;
    this._idx.set(obj, idx);
    this._typs.push(buf);
  }

  public merge(obj: string, knot: string) {
    const idx = this._idx.get(obj);
    const knotIdx = this._idx.get(knot);
    if (idx === undefined) {
      throw new Error("Missing type index for " + obj);
    }
    if (knotIdx === undefined) {
      throw new Error("Missing type index for " + knot);
    }
    this._typs[idx] = this._typs[knotIdx];

    // Delete the type.
    this._typs.splice(knotIdx, 1);
    this._idx.delete(knot);
  }

  public encode() {
    const len = leb.encode(this._typs.length);
    const buf = Buffer.concat(this._typs);
    return Buffer.concat([len, buf]);
  }

  public indexOf(typeName: string) {
    if (!this._idx.has(typeName)) {
      throw new Error("Missing type index for " + typeName);
    }
    return sleb.encode(this._idx.get(typeName));
  }
}

/**
 * Represents an IDL type.
 */
export abstract class Type<T = any> {

  public abstract readonly name: string;
  /* Memoized DFS for storing type description into TypeTable  */
  public buildType(t: TypeTable): void {
    if (!t.has(this.name)) {
      this.buildTypeGo(t);
    }
  }

  /* Implement T in the IDL spec, only needed for non-primitive types */
  public buildTypeGo(typeTable: TypeTable): void {}

  public validate(x: any): x is T {
    throw new Error("You have the implement the method validate.");
  }

  public encode(x: any): Buffer {
    if (this.validate(x)) {
      return this.encodeGo(x);
    }

    throw new Error(`Invalid ${this.name} argument: "${x}"`);
  }

  public encodeGo(x: T): Buffer {
    throw new Error("You have to implement the method encodeGo!");
  }

  /* Implement I in the IDL spec */
  public encodeTypeGo(typeTable: TypeTable): Buffer {
    throw new Error("You have to implement the method encodeTypeGo!");
  }

  public decodeGo(x: Pipe): T {
    throw new Error("You have to implement the method decodeGo!");
  }
}

/**
 * Represents an IDL None, a type which has no inhabitants.
 * Since no values exist for this type, it cannot be serialised or deserialised.
 * Result types like `Result<Text, None>` should always succeed.
 */
export class NoneClass extends Type<never> {
  public encodeGo(): never {
    throw new Error("None cannot appear as a function argument");
  }

  public encodeTypeGo() {
    return sleb.encode(-17);
  }

  public decodeGo(): never {
    throw new Error("None cannot appear as an output");
  }

  get name() {
    return "None";
  }
}

/**
 * Represents an IDL Bool
 */
export class BoolClass extends Type<boolean> {
  public validate(x: any): x is boolean {
    return typeof x === "boolean";
  }

  public encodeGo(x: boolean): Buffer {
    const buf = Buffer.alloc(1);
    buf.writeInt8(x ? 1 : 0, 0);
    return buf;
  }

  public encodeTypeGo() {
    return sleb.encode(-2);
  }

  public decodeGo(b: Pipe) {
    const x = b.read(1).toString("hex");
    return x === "01";
  }

  get name() {
    return "Bool";
  }
}

/**
 * Represents an IDL Unit
 */
export class UnitClass extends Type<null> {
  public encodeGo() {
    return Buffer.alloc(0);
  }

  public encodeTypeGo() {
    return sleb.encode(-1);
  }

  public decodeGo() {
    return null;
  }

  get name() {
    return "Unit";
  }
}

/**
 * Represents an IDL Text
 */
export class TextClass extends Type<string> {
  public validate(x: any): x is string {
    return typeof x === "string";
  }

  public encodeGo(x: string) {
    const buf = Buffer.from(x, "utf8");
    const len = leb.encode(buf.length);
    return Buffer.concat([len, buf]);
  }

  public encodeTypeGo() {
    return sleb.encode(-15);
  }

  public decodeGo(b: Pipe) {
    const len = leb.readBn(b).toNumber();
    const x = b.read(len).toString("utf8");
    return x;
  }

  get name() {
    return "Text";
  }
}

/**
 * Represents an IDL Int
 */
export class IntClass extends Type<number> {
  public validate(x: any): x is number {
    return Number.isInteger(x);
  }

  public encodeGo(x: number) {
    return sleb.encode(x);
  }

  public encodeTypeGo() {
    return sleb.encode(-4);
  }

  public decodeGo(b: Pipe) {
    return sleb.readBn(b).toNumber();
  }

  get name() {
    return "Int";
  }
}

/**
 * Represents an IDL Nat
 */
export class NatClass extends Type {
  public validate(x: any): x is number {
    return Number.isInteger(x) && x >= 0;
  }

  public encodeGo(x: number) {
    return leb.encode(x);
  }

  public encodeTypeGo() {
    return sleb.encode(-3);
  }

  public decodeGo(b: Pipe) {
    return leb.readBn(b).toNumber();
  }

  get name() {
    return "Nat";
  }
}

/**
 * Represents an IDL Tuple
 * @param {Type} components
 */
export class TupleClass<T extends any[]> extends Type<T> {
  constructor(private _components: Type[]) {
    super();
  }

  public validate(x: any): x is T {
    return Array.isArray(x) && x.length === this._components.length;
  }

  public encodeGo(x: any[]) {
    const bufs = zipWith(this._components, x, (c, d) => c.encode(d));
    return Buffer.concat(bufs);
  }

  public buildTypeGo(typeTable: TypeTable) {
    const components = this._components;
    components.forEach((x) => x.buildType(typeTable));

    const opCode = sleb.encode(-20);
    const len = leb.encode(components.length);
    const buf = Buffer.concat(
      components.map((x, i) =>
        Buffer.concat([leb.encode(i), x.encodeTypeGo(typeTable)]),
      ),
    );
    typeTable.add(this.name, Buffer.concat([opCode, len, buf]));
  }

  public encodeTypeGo(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }

  public decodeGo(b: Pipe): T {
    return this._components.map((c) => c.decodeGo(b)) as T;
  }

  get name() {
    return `Tuple(${this._components.map((x) => x.name).join(",")})`;
  }
}

/**
 * Represents an IDL Array
 * @param {Type} t
 */
export class ArrClass extends Type<any[]> {
  constructor(private _type: Type) {
    super();
  }

  public encodeGo(x: any[]) {
    if (!Array.isArray(x)) {
      throw Error("Invalid Arr argument: " + x);
    }
    const len = leb.encode(x.length);
    const xs = x.map((d) => this._type.encode(d));
    return Buffer.concat([len, ...xs]);
  }

  public buildTypeGo(T: TypeTable) {
    this._type.buildType(T);
    const opCode = sleb.encode(-19);
    const buffer = this._type.encodeTypeGo(T);
    T.add(this.name, Buffer.concat([opCode, buffer]));
  }

  public encodeTypeGo(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }

  public decodeGo(b: Pipe): any[] {
    const len = leb.readBn(b).toNumber();
    const rets: any[] = [];
    for (let i = 0; i < len; i++) {
      rets.push(this._type.decodeGo(b));
    }
    return rets;
  }

  get name() {
    return `Arr(${this._type.name})`;
  }
}

/**
 * Represents an IDL Option
 * @param {Type} t
 */
export class OptClass extends Type {
  constructor(private _type: Type) {
    super();
  }

  public encodeGo(x: any) {
    if (x == null) {
      return Buffer.from([0]);
    } else {
      return Buffer.concat([Buffer.from([1]), this._type.encode(x)]);
    }
  }

  public buildTypeGo(T: TypeTable) {
    this._type.buildType(T);
    const opCode = sleb.encode(-18);
    const buffer = this._type.encodeTypeGo(T);
    T.add(this.name, Buffer.concat([opCode, buffer]));
  }

  public encodeTypeGo(T: TypeTable) {
    return T.indexOf(this.name);
  }

  public decodeGo(b: Pipe) {
    const len = b.read(1).toString("hex");
    if (len === "00") {
      return null;
    } else {
      return this._type.decodeGo(b);
    }
  }

  get name() {
    return `Opt(${this._type.name})`;
  }
}

/**
 * Represents an IDL Object
 * @param {Object} [fields] - mapping of function name to Type
 */
export class ObjClass extends Type<Record<string, Type>> {
  protected _fields: Array<[string, Type]>;

  constructor(fields: Record<string, Type> = {}) {
    super();
    const sortedFields = Object.entries(fields).sort(
      (a, b) => hash(a[0]) - hash(b[0]),
    );

    this._fields = sortedFields;
  }

  public encodeGo(x: Record<string, any>) {
    const values = this._fields.map(([key, _]) => {
      if (!x.hasOwnProperty(key)) {
        throw Error("Obj is missing key: " + key);
      }
      return x[key];
    });
    const bufs = zipWith(this._fields, values, ([_, c], d) => c.encode(d));
    return Buffer.concat(bufs);
  }

  public buildTypeGo(T: TypeTable) {
    this._fields.forEach(([_, value]) => value.buildType(T));
    const opCode = sleb.encode(-20);
    const len = leb.encode(this._fields.length);
    const fields = this._fields.map(([key, value]) =>
      Buffer.concat([leb.encode(hash(key)), value.encodeTypeGo(T)]),
    );

    T.add(this.name, Buffer.concat([opCode, len, Buffer.concat(fields)]));
  }

  public encodeTypeGo(T: TypeTable) {
    return T.indexOf(this.name);
  }

  public decodeGo(b: Pipe) {
    const x: Record<string, any> = {};
    for (const [key, value] of this._fields) {
      x[key] = value.decodeGo(b);
    }
    return x;
  }

  get name() {
    const fields = this._fields.map(([key, value]) => key + ":" + value);
    return `Obj(${fields.join(",")})`;
  }
}

/**
 * Represents an IDL Variant
 * @param {Object} [fields] - mapping of function name to Type
 */
export class VariantClass extends Type {
  private readonly _fields: Array<[string, Type]>;

  constructor(fields: Record<string, Type> = {}) {
    super();
    this._fields = Object.entries(fields).sort(
      (a, b) => hash(a[0]) - hash(b[0]),
    );
  }

  public encodeGo(x: any) {
    let out: Buffer | undefined;
    for (let i = 0; i < this._fields.length; i++) {
      const [k, v] = this._fields[i];
      if (x.hasOwnProperty(k)) {
        if (out) {
          throw Error("Variant has extra key: " + k);
        }
        const idx = leb.encode(i);
        const buf = v.encode(x[k]);
        out = Buffer.concat([idx, buf]);
      }
    }
    if (!out) {
      throw Error("Variant has no data: " + x);
    }
    return out;
  }

  public buildTypeGo(T: TypeTable) {
    this._fields.forEach(([key, value]) => value.buildType(T));
    const opCode = sleb.encode(-21);
    const len = leb.encode(this._fields.length);
    const fields = this._fields.map(([key, value]) =>
      Buffer.concat([leb.encode(hash(key)), value.encodeTypeGo(T)]),
    );
    T.add(this.name, Buffer.concat([opCode, len, Buffer.concat(fields)]));
  }

  public encodeTypeGo(T: TypeTable) {
    return T.indexOf(this.name);
  }

  public decodeGo(b: Pipe) {
    const idx = leb.readBn(b).toNumber();
    if (idx >= this._fields.length) {
      throw Error("Invalid variant: " + idx);
    }

    const value = this._fields[idx][1].decodeGo(b);
    return {
      [this._fields[idx][0]]: value,
    };
  }

  get name() {
    const fields = this._fields.map(([key, value]) => key + ":" + value);
    return `Variant(${fields})`;
  }
}

/**
 * Represents a reference to an IDL type, used for defining recursive data
 * types.
 */
export class RecClass extends Type {
  private static _counter = 0;
  private _id = RecClass._counter++;
  private _type: Type | undefined = undefined;

  public fill(t: Type) {
    this._type = t;
  }

  public validate(x: any): x is any {
    return this._type ? this._type.validate(x) : false;
  }

  public encodeGo(x: any) {
    if (!this._type) {
      throw Error("Recursive type uninitialized.");
    }
    return this._type.encode(x);
  }

  public buildTypeGo(T: TypeTable) {
    if (!this._type) {
      throw Error("Recursive type uninitialized.");
    }
    T.add(this.name, Buffer.alloc(0));
    this._type.buildType(T);
    T.merge(this.name, this._type.name);
  }

  public encodeTypeGo(T: TypeTable) {
    return T.indexOf(this.name);
  }

  public decodeGo(b: Pipe) {
    if (!this._type) {
      throw Error("Recursive type uninitialized.");
    }
    return this._type.decodeGo(b);
  }

  get name() {
    return `Rec(${this._id})`;
  }
}

/**
 * Represents an async function which can return data
 * @param {Array<Type>} [argTypes] - argument types
 * @param {Array<Type>} [retTypes] - return types
 */
export class FuncClass extends Type<any> {
  public argTypes: Type[];
  public retTypes: Type[];

  constructor(argTypes: Type[] = [], retTypes: Type[]) {
    super();

    if (!Array.isArray(argTypes)) {
      throw Error(
        "First argument to Func must be an array of IDL argument types.",
      );
    }
    if (retTypes && !Array.isArray(retTypes)) {
      throw Error(
        "Second argument to Func must be an array of IDL argument types.",
      );
    }
    this.argTypes = argTypes;
    this.retTypes = retTypes;
  }

  get name() {
    const ret = this.retTypes.map((x) => x.name);
    return `Func(${this.argTypes.map((x) => x.name).join(",")}):${ret.join(",")}`;
  }
}

/**
 * Encode a array of values
 * @returns {Buffer} serialised value
 */
export function encode(argTypes: Array<Type<any>>, args: any[]) {
  if (args.length !== argTypes.length) {
    throw Error("Wrong number of message arguments");
  }
  const T = new TypeTable();
  argTypes.forEach((t) => t.buildType(T));

  const magic = Buffer.from(magicNumber, "utf8");
  const table = T.encode();
  const len = leb.encode(args.length);
  const typs = Buffer.concat(argTypes.map((t) => t.encodeTypeGo(T)));
  const vals = Buffer.concat(zipWith(argTypes, args, (t, x) => t.encode(x)));

  return Buffer.concat([magic, table, len, typs, vals]);
}

/**
 * Decode a binary value
 * @param retTypes - Types expected in the buffer.
 * @param bytes - hex-encoded string, or buffer.
 * @returns Value deserialised to JS type
 */
export function decode(retTypes: Type[], bytes: Buffer): JsonValue[] {
  const b = new Pipe(bytes);

  if (bytes.byteLength < magicNumber.length) {
    throw new Error("Message length smaller than magic number");
  }
  const magic = b.read(magicNumber.length).toString();
  if (magic !== magicNumber) {
    throw new Error("Wrong magic number: " + magic);
  }

  function decodeType(pipe: Pipe) {
    const len = leb.readBn(pipe).toNumber();

    for (let i = 0; i < len; i++) {
      const ty = sleb.readBn(pipe).toNumber();
      switch (ty) {
        case -18: // opt
          sleb.readBn(pipe).toNumber();
          break;
        case -19: // vec
          sleb.readBn(pipe).toNumber();
          break;
        case -20: { // record/tuple
          let objectLength = leb.readBn(pipe).toNumber();
          while (objectLength--) {
            leb.readBn(pipe).toNumber();
            sleb.readBn(pipe).toNumber();
          }
          break;
        }
        case -21: { // variant
          let variantLength = leb.readBn(pipe).toNumber();
          while (variantLength--) {
            leb.readBn(pipe).toNumber();
            sleb.readBn(pipe).toNumber();
          }
          break;
        }
        default:
          throw new Error("Illegal op_code: " + ty);
      }
    }

    const length = leb.readBn(pipe);
    for (let i = 0; i < length; i++) {
      sleb.readBn(pipe).toNumber();
    }
  }

  decodeType(b);
  const output = retTypes.map((t) => t.decodeGo(b));
  if (b.buffer.length > 0) {
    throw new Error("decode: Left-over bytes");
  }

  return output;
}

/**
 * A wrapper over a client and an IDL
 * @param {Object} [fields] - a map of function names to IDL function signatures
 */
export class ActorInterface {
  protected _id: Blob | null = null;
  protected _batch: boolean = false;

  constructor(public _fields: Record<string, Type>) {}
}

// Export Types instances.
export const None = new NoneClass();
export const Bool = new BoolClass();
export const Unit = new UnitClass();
export const Text = new TextClass();
export const Int = new IntClass();
export const Nat = new NatClass();

export function Tuple<T extends any[]>(...types: T): TupleClass<T> {
  return new TupleClass(types);
}
export function Arr(t: Type): ArrClass {
  return new ArrClass(t);
}
export function Opt(t: Type): OptClass {
  return new OptClass(t);
}
export function Obj(t: Record<string, Type>): ObjClass {
  return new ObjClass(t);
}
export function Variant(fields: Record<string, Type> = {}) {
  return new VariantClass(fields);
}
export function Rec() {
  return new RecClass();
}

export function Func(args: Type[], ret: Type[]) {
  return new FuncClass(args, ret);
}
