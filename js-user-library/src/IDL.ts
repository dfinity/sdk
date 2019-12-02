// tslint:disable:max-classes-per-file
import { Buffer } from 'buffer';
import Pipe = require('buffer-pipe');
import { signed as sleb, unsigned as leb } from 'leb128';

// tslint:disable:max-line-length
/**
 * This module provides a combinator library to create serializers/deserializers
 * between JavaScript values and IDL used by canisters on the Internet Computer,
 * as documented at https://github.com/dfinity-lab/motoko/blob/2f71cfc9590741425db752a029e0758f94284e79/design/IDL.md
 */
// tslint:enable:max-line-length

function zipWith<TX, TY, TR>(xs: TX[], ys: TY[], f: (a: TX, b: TY) => TR): TR[] {
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

const magicNumber = 'DIDL';

export class TypeTable {
  // List of types. Needs to be an array as the index needs to be stable.
  private _typs: Buffer[] = [];
  private _idx = new Map<string, number>();

  public has(obj: Type) {
    return this._idx.has(obj.name);
  }

  public add<T>(type: Type<T>, buf: Buffer) {
    if (type instanceof PrimitiveType) {
      throw new Error('TypeTable cannot contain primitive types.');
    }

    const idx = this._typs.length;
    this._idx.set(type.name, idx);
    this._typs.push(buf);
  }

  public merge<T>(obj: Type<T>, knot: string) {
    const idx = this._idx.get(obj.name);
    const knotIdx = this._idx.get(knot);
    if (idx === undefined) {
      throw new Error('Missing type index for ' + obj);
    }
    if (knotIdx === undefined) {
      throw new Error('Missing type index for ' + knot);
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
      throw new Error('Missing type index for ' + typeName);
    }
    return sleb.encode(this._idx.get(typeName));
  }
}

/**
 * Represents an IDL type.
 */
export abstract class Type<T = any> {
  public abstract readonly name: string;

  /* Implement `T` in the IDL spec, only needed for non-primitive types */
  public buildTypeTable(typeTable: TypeTable): void {
    if (!typeTable.has(this)) {
      this._buildTypeTableImpl(typeTable);
    }
  }

  /**
   * Assert that JavaScript's `x` is the proper type represented by this
   * Type.
   */
  public abstract covariant(x: any): x is T;

  /**
   * Encode the value. This needs to be public because it is used by
   * encodeValue() from different types.
   * @internal
   */
  public abstract encodeValue(x: T): Buffer;

  /**
   * Implement `I` in the IDL spec.
   * Encode this type for the type table.
   */
  public abstract encodeType(typeTable: TypeTable): Buffer;

  public abstract decodeValue(x: Pipe): T;

  protected abstract _buildTypeTableImpl(typeTable: TypeTable): void;
}

export abstract class PrimitiveType<T = any> extends Type<T> {
  public _buildTypeTableImpl(typeTable: TypeTable): void {
    // No type table encoding for Primitive types.
    return;
  }
}

/**
 * Represents an IDL None, a type which has no inhabitants.
 * Since no values exist for this type, it cannot be serialised or deserialised.
 * Result types like `Result<Text, None>` should always succeed.
 */
export class NoneClass extends PrimitiveType<never> {
  public covariant(x: any): x is never {
    return false;
  }

  public encodeValue(): never {
    throw new Error('None cannot appear as a function argument');
  }

  public encodeType() {
    return sleb.encode(-17);
  }

  public decodeValue(): never {
    throw new Error('None cannot appear as an output');
  }

  get name() {
    return 'None';
  }
}

/**
 * Represents an IDL Bool
 */
export class BoolClass extends PrimitiveType<boolean> {
  public covariant(x: any): x is boolean {
    return typeof x === 'boolean';
  }

  public encodeValue(x: boolean): Buffer {
    const buf = Buffer.alloc(1);
    buf.writeInt8(x ? 1 : 0, 0);
    return buf;
  }

  public encodeType() {
    return sleb.encode(-2);
  }

  public decodeValue(b: Pipe) {
    const x = b.read(1).toString('hex');
    return x === '01';
  }

  get name() {
    return 'Bool';
  }
}

/**
 * Represents an IDL Unit
 */
export class UnitClass extends PrimitiveType<null> {
  public covariant(x: any): x is null {
    return x === null;
  }

  public encodeValue() {
    return Buffer.alloc(0);
  }

  public encodeType() {
    return sleb.encode(-1);
  }

  public decodeValue() {
    return null;
  }

  get name() {
    return 'Unit';
  }
}

/**
 * Represents an IDL Text
 */
export class TextClass extends PrimitiveType<string> {
  public covariant(x: any): x is string {
    return typeof x === 'string';
  }

  public encodeValue(x: string) {
    const buf = Buffer.from(x, 'utf8');
    const len = leb.encode(buf.length);
    return Buffer.concat([len, buf]);
  }

  public encodeType() {
    return sleb.encode(-15);
  }

  public decodeValue(b: Pipe) {
    const len = leb.readBn(b).toNumber();
    return b.read(len).toString('utf8');
  }

  get name() {
    return 'Text';
  }
}

/**
 * Represents an IDL Int
 */
export class IntClass extends PrimitiveType<number> {
  public covariant(x: any): x is number {
    return Number.isInteger(x);
  }

  public encodeValue(x: number) {
    return sleb.encode(x);
  }

  public encodeType() {
    return sleb.encode(-4);
  }

  public decodeValue(b: Pipe) {
    return sleb.readBn(b).toNumber();
  }

  get name() {
    return 'Int';
  }
}

/**
 * Represents an IDL Nat
 */
export class NatClass extends PrimitiveType<number> {
  public covariant(x: any): x is number {
    return Number.isInteger(x) && x >= 0;
  }

  public encodeValue(x: number) {
    return leb.encode(x);
  }

  public encodeType() {
    return sleb.encode(-3);
  }

  public decodeValue(b: Pipe) {
    return leb.readBn(b).toNumber();
  }

  get name() {
    return 'Nat';
  }
}

/**
 * Represents an IDL Tuple, a Record that has the index as the key.
 * @param {Type} components
 */
export class TupleClass<T extends any[]> extends Type<T> {
  constructor(private _components: Type[]) {
    super();
  }

  public covariant(x: any): x is T {
    // `>=` because tuples can be covariant when encoded.
    return (
      Array.isArray(x) &&
      x.length >= this._components.length &&
      this._components.every((t, i) => t.covariant(x[i]))
    );
  }

  public encodeValue(x: any[]) {
    const bufs = zipWith(this._components, x, (c, d) => c.encodeValue(d));
    return Buffer.concat(bufs);
  }

  public _buildTypeTableImpl(typeTable: TypeTable) {
    const components = this._components;
    components.forEach(x => x.buildTypeTable(typeTable));

    const opCode = sleb.encode(-20);
    const len = leb.encode(components.length);
    const buf = Buffer.concat(
      components.map((x, i) => {
        return Buffer.concat([leb.encode(i), x.encodeType(typeTable)]);
      }),
    );
    typeTable.add(this, Buffer.concat([opCode, len, buf]));
  }

  public encodeType(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }

  public decodeValue(b: Pipe): T {
    return this._components.map(c => c.decodeValue(b)) as T;
  }

  get name() {
    return `Tuple(${this._components.map(x => x.name).join(',')})`;
  }
}

/**
 * Represents an IDL Array
 * @param {Type} t
 */
export class ArrClass<T> extends Type<T[]> {
  constructor(protected _type: Type<T>) {
    super();
  }

  public covariant(x: any): x is T[] {
    return Array.isArray(x) && x.every(v => this._type.covariant(v));
  }

  public encodeValue(x: T[]) {
    const len = leb.encode(x.length);
    return Buffer.concat([len, ...x.map(d => this._type.encodeValue(d))]);
  }

  public _buildTypeTableImpl(typeTable: TypeTable) {
    this._type.buildTypeTable(typeTable);

    const opCode = sleb.encode(-19);
    const buffer = this._type.encodeType(typeTable);
    typeTable.add(this, Buffer.concat([opCode, buffer]));
  }

  public encodeType(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }

  public decodeValue(b: Pipe): any[] {
    const len = leb.readBn(b).toNumber();
    const rets: any[] = [];
    for (let i = 0; i < len; i++) {
      rets.push(this._type.decodeValue(b));
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
export class OptClass<T> extends Type<T | null> {
  constructor(protected _type: Type<T>) {
    super();
  }

  public covariant(x: any): x is T | null {
    return x == null || this._type.covariant(x);
  }

  public encodeValue(x: T | null) {
    if (x === null) {
      return Buffer.from([0]);
    } else {
      return Buffer.concat([Buffer.from([1]), this._type.encodeValue(x)]);
    }
  }

  public _buildTypeTableImpl(typeTable: TypeTable) {
    this._type.buildTypeTable(typeTable);

    const opCode = sleb.encode(-18);
    const buffer = this._type.encodeType(typeTable);
    typeTable.add(this, Buffer.concat([opCode, buffer]));
  }

  public encodeType(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }

  public decodeValue(b: Pipe): T | null {
    const len = b.read(1).toString('hex');
    if (len === '00') {
      return null;
    } else {
      return this._type.decodeValue(b);
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
export class ObjClass extends Type<Record<string, any>> {
  protected readonly _fields: Array<[string, Type]>;

  constructor(fields: Record<string, Type> = {}) {
    super();
    this._fields = Object.entries(fields).sort((a, b) => hash(a[0]) - hash(b[0]));
  }

  public covariant(x: any): x is Record<string, any> {
    return (
      typeof x === 'object' &&
      this._fields.every(([k, t]) => {
        if (!x.hasOwnProperty(k)) {
          throw new Error(`Obj is missing key "${k}".`);
        }
        return t.covariant(x[k]);
      })
    );
  }

  public encodeValue(x: Record<string, any>) {
    const values = this._fields.map(([key]) => x[key]);
    const bufs = zipWith(this._fields, values, ([_, c], d) => c.encodeValue(d));
    return Buffer.concat(bufs);
  }

  public _buildTypeTableImpl(T: TypeTable) {
    this._fields.forEach(([, value]) => value.buildTypeTable(T));
    const opCode = sleb.encode(-20);
    const len = leb.encode(this._fields.length);
    const fields = this._fields.map(([key, value]) =>
      Buffer.concat([leb.encode(hash(key)), value.encodeType(T)]),
    );

    T.add(this, Buffer.concat([opCode, len, Buffer.concat(fields)]));
  }

  public encodeType(T: TypeTable) {
    return T.indexOf(this.name);
  }

  public decodeValue(b: Pipe) {
    const x: Record<string, any> = {};
    for (const [key, value] of this._fields) {
      x[key] = value.decodeValue(b);
    }
    return x;
  }

  get name() {
    const fields = this._fields.map(([key, value]) => key + ':' + value.name);
    return `Obj(${fields.join(',')})`;
  }
}

/**
 * Represents an IDL Variant
 * @param {Object} [fields] - mapping of function name to Type
 */
export class VariantClass extends Type<Record<string, any>> {
  private readonly _fields: Array<[string, Type]>;

  constructor(fields: Record<string, Type> = {}) {
    super();
    this._fields = Object.entries(fields).sort((a, b) => hash(a[0]) - hash(b[0]));
  }

  public covariant(x: any): x is Record<string, any> {
    return (
      typeof x === 'object' &&
      Object.entries(x).length === 1 &&
      this._fields.every(([k, v]) => {
        return !x.hasOwnProperty(k) || v.covariant(x[k]);
      })
    );
  }

  public encodeValue(x: Record<string, any>) {
    for (let i = 0; i < this._fields.length; i++) {
      const [name, type] = this._fields[i];
      if (x.hasOwnProperty(name)) {
        const idx = leb.encode(i);
        const buf = type.encodeValue(x[name]);

        return Buffer.concat([idx, buf]);
      }
    }
    throw Error('Variant has no data: ' + x);
  }

  public _buildTypeTableImpl(typeTable: TypeTable) {
    this._fields.forEach(([, type]) => {
      type.buildTypeTable(typeTable);
    });
    const opCode = sleb.encode(-21);
    const len = leb.encode(this._fields.length);
    const fields = this._fields.map(([key, value]) =>
      Buffer.concat([leb.encode(hash(key)), value.encodeType(typeTable)]),
    );
    typeTable.add(this, Buffer.concat([opCode, len, ...fields]));
  }

  public encodeType(T: TypeTable) {
    return T.indexOf(this.name);
  }

  public decodeValue(b: Pipe) {
    const idx = leb.readBn(b).toNumber();
    if (idx >= this._fields.length) {
      throw Error('Invalid variant: ' + idx);
    }

    const value = this._fields[idx][1].decodeValue(b);
    return {
      [this._fields[idx][0]]: value,
    };
  }

  get name() {
    const fields = this._fields.map(([key, type]) => key + ':' + type.name);
    return `Variant(${fields.join(',')})`;
  }
}

/**
 * Represents a reference to an IDL type, used for defining recursive data
 * types.
 */
export class RecClass<T = any> extends Type<T> {
  private static _counter = 0;
  private _id = RecClass._counter++;
  private _type: Type<T> | undefined = undefined;

  public fill(t: Type<T>) {
    this._type = t;
  }

  public covariant(x: any): x is T {
    return this._type ? this._type.covariant(x) : false;
  }

  public encodeValue(x: T) {
    if (!this._type) {
      throw Error('Recursive type uninitialized.');
    }
    return this._type.encodeValue(x);
  }

  public _buildTypeTableImpl(typeTable: TypeTable) {
    if (!this._type) {
      throw Error('Recursive type uninitialized.');
    }
    typeTable.add(this, Buffer.alloc(0));
    this._type.buildTypeTable(typeTable);
    typeTable.merge(this, this._type.name);
  }

  public encodeType(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }

  public decodeValue(b: Pipe) {
    if (!this._type) {
      throw Error('Recursive type uninitialized.');
    }
    return this._type.decodeValue(b);
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
export class FuncClass {
  constructor(public argTypes: Type[] = [], public retTypes: Type[] = []) {}

  get name() {
    const ret = this.retTypes.map(x => x.name);
    return `Func(${this.argTypes.map(x => x.name).join(',')}):${ret.join(',')}`;
  }
}

/**
 * Encode a array of values
 * @returns {Buffer} serialised value
 */
export function encode(argTypes: Array<Type<any>>, args: any[]) {
  if (args.length < argTypes.length) {
    throw Error('Wrong number of message arguments');
  }

  const typeTable = new TypeTable();
  argTypes.forEach(t => t.buildTypeTable(typeTable));

  const magic = Buffer.from(magicNumber, 'utf8');
  const table = typeTable.encode();
  const len = leb.encode(args.length);
  const typs = Buffer.concat(argTypes.map(t => t.encodeType(typeTable)));
  const vals = Buffer.concat(
    zipWith(argTypes, args, (t, x) => {
      if (!t.covariant(x)) {
        throw new Error(`Invalid ${t.name} argument: "${JSON.stringify(x)}"`);
      }

      return t.encodeValue(x);
    }),
  );

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
    throw new Error('Message length smaller than magic number');
  }
  const magic = b.read(magicNumber.length).toString();
  if (magic !== magicNumber) {
    throw new Error('Wrong magic number: ' + magic);
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
        case -20: {
          // record/tuple
          let objectLength = leb.readBn(pipe).toNumber();
          while (objectLength--) {
            leb.readBn(pipe).toNumber();
            sleb.readBn(pipe).toNumber();
          }
          break;
        }
        case -21: {
          // variant
          let variantLength = leb.readBn(pipe).toNumber();
          while (variantLength--) {
            leb.readBn(pipe).toNumber();
            sleb.readBn(pipe).toNumber();
          }
          break;
        }
        default:
          throw new Error('Illegal op_code: ' + ty);
      }
    }

    const length = leb.readBn(pipe);
    for (let i = 0; i < length; i++) {
      sleb.readBn(pipe).toNumber();
    }
  }

  decodeType(b);
  const output = retTypes.map(t => t.decodeValue(b));
  if (b.buffer.length > 0) {
    throw new Error('decode: Left-over bytes');
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

  constructor(public _fields: Record<string, FuncClass>) {}
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
export function Arr<T>(t: Type<T>): ArrClass<T> {
  return new ArrClass(t);
}
export function Opt<T>(t: Type<T>): OptClass<T> {
  return new OptClass(t);
}
export function Obj(t: Record<string, Type>): ObjClass {
  return new ObjClass(t);
}
export function Variant(fields: Record<string, Type>) {
  return new VariantClass(fields);
}
export function Rec() {
  return new RecClass();
}

export function Func(args: Type[], ret: Type[]) {
  return new FuncClass(args, ret);
}
