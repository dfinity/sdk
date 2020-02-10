// tslint:disable:max-classes-per-file
import BigNumber from 'bignumber.js';
import Pipe = require('buffer-pipe');
import { Buffer } from 'buffer/';
import { JsonValue } from './types';
import { idlLabelToId } from './utils/hash';
import { lebDecode, lebEncode, slebDecode, slebEncode } from './utils/leb128';
import { readIntLE, readUIntLE, writeIntLE, writeUIntLE } from './utils/leb128';

// tslint:disable:max-line-length
/**
 * This module provides a combinator library to create serializers/deserializers
 * between JavaScript values and IDL used by canisters on the Internet Computer,
 * as documented at https://github.com/dfinity-lab/motoko/blob/2f71cfc9590741425db752a029e0758f94284e79/design/IDL.md
 */
// tslint:enable:max-line-length

const enum IDLTypeIds {
  Null = -1,
  Bool = -2,
  Nat = -3,
  Int = -4,
  Text = -15,
  Empty = -17,
  Opt = -18,
  Vector = -19,
  Record = -20,
  Variant = -21,
}

const magicNumber = 'DIDL';

function zipWith<TX, TY, TR>(xs: TX[], ys: TY[], f: (a: TX, b: TY) => TR): TR[] {
  return xs.map((x, i) => f(x, ys[i]));
}

/**
 * An IDL Type Table, which precedes the data in the stream.
 */
class TypeTable {
  // List of types. Needs to be an array as the index needs to be stable.
  private _typs: Buffer[] = [];
  private _idx = new Map<string, number>();

  public has(obj: ConstructType) {
    return this._idx.has(obj.name);
  }

  public add<T>(type: ConstructType<T>, buf: Buffer) {
    const idx = this._typs.length;
    this._idx.set(type.name, idx);
    this._typs.push(buf);
  }

  public merge<T>(obj: ConstructType<T>, knot: string) {
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
    const len = lebEncode(this._typs.length);
    const buf = Buffer.concat(this._typs);
    return Buffer.concat([len, buf]);
  }

  public indexOf(typeName: string) {
    if (!this._idx.has(typeName)) {
      throw new Error('Missing type index for ' + typeName);
    }
    return slebEncode(this._idx.get(typeName) || 0);
  }
}

/**
 * Represents an IDL type.
 */
export abstract class Type<T = any> {
  public abstract readonly name: string;

  /* Display type name */
  public display(): string {
    return this.name;
  }

  public valueToString(x: T): string {
    return JSON.stringify(x);
  }

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

abstract class PrimitiveType<T = any> extends Type<T> {
  public _buildTypeTableImpl(typeTable: TypeTable): void {
    // No type table encoding for Primitive types.
    return;
  }
}

abstract class ConstructType<T = any> extends Type<T> {
  public encodeType(typeTable: TypeTable) {
    return typeTable.indexOf(this.name);
  }
}

/**
 * Represents an IDL Empty, a type which has no inhabitants.
 * Since no values exist for this type, it cannot be serialised or deserialised.
 * Result types like `Result<Text, Empty>` should always succeed.
 */
class EmptyClass extends PrimitiveType<never> {
  public covariant(x: any): x is never {
    return false;
  }

  public encodeValue(): never {
    throw new Error('Empty cannot appear as a function argument');
  }

  public valueToString(): never {
    throw new Error('Empty cannot appear as a value');
  }

  public encodeType() {
    return slebEncode(IDLTypeIds.Empty);
  }

  public decodeValue(): never {
    throw new Error('Empty cannot appear as an output');
  }

  get name() {
    return 'empty';
  }
}

/**
 * Represents an IDL Bool
 */
class BoolClass extends PrimitiveType<boolean> {
  public covariant(x: any): x is boolean {
    return typeof x === 'boolean';
  }

  public encodeValue(x: boolean): Buffer {
    const buf = Buffer.alloc(1);
    buf.writeInt8(x ? 1 : 0, 0);
    return buf;
  }

  public encodeType() {
    return slebEncode(IDLTypeIds.Bool);
  }

  public decodeValue(b: Pipe) {
    const x = b.read(1).toString('hex');
    return x === '01';
  }

  get name() {
    return 'bool';
  }
}

/**
 * Represents an IDL Null
 */
class UnitClass extends PrimitiveType<null> {
  public covariant(x: any): x is null {
    return x === null;
  }

  public encodeValue() {
    return Buffer.alloc(0);
  }

  public encodeType() {
    return slebEncode(IDLTypeIds.Null);
  }

  public decodeValue() {
    return null;
  }

  get name() {
    return 'null';
  }
}

/**
 * Represents an IDL Text
 */
class TextClass extends PrimitiveType<string> {
  public covariant(x: any): x is string {
    return typeof x === 'string';
  }

  public encodeValue(x: string) {
    const buf = Buffer.from(x, 'utf8');
    const len = lebEncode(buf.length);
    return Buffer.concat([len, buf]);
  }

  public encodeType() {
    return slebEncode(IDLTypeIds.Text);
  }

  public decodeValue(b: Pipe) {
    const len = lebDecode(b).toNumber();
    return b.read(len).toString('utf8');
  }

  get name() {
    return 'text';
  }
}

/**
 * Represents an IDL Int
 */
class IntClass extends PrimitiveType<BigNumber> {
  public covariant(x: any): x is BigNumber {
    // We allow encoding of JavaScript plain numbers.
    // But we will always decode to BigNumber.
    return (x instanceof BigNumber && x.isInteger()) || Number.isInteger(x);
  }

  public encodeValue(x: BigNumber | number) {
    return slebEncode(x);
  }

  public encodeType() {
    return slebEncode(IDLTypeIds.Int);
  }

  public decodeValue(b: Pipe) {
    return slebDecode(b);
  }

  get name() {
    return 'int';
  }

  public valueToString(x: BigNumber) {
    return x.toFixed();
  }
}

/**
 * Represents an IDL Nat
 */
class NatClass extends PrimitiveType<BigNumber> {
  public covariant(x: any): x is BigNumber {
    // We allow encoding of JavaScript plain numbers.
    // But we will always decode to BigNumber.
    return (
      (x instanceof BigNumber && x.isInteger() && !x.isNegative()) ||
      (Number.isInteger(x) && x >= 0)
    );
  }

  public encodeValue(x: BigNumber | number) {
    return lebEncode(x);
  }

  public encodeType() {
    return slebEncode(IDLTypeIds.Nat);
  }

  public decodeValue(b: Pipe) {
    return lebDecode(b);
  }

  get name() {
    return 'nat';
  }

  public valueToString(x: BigNumber) {
    return x.toFixed();
  }
}

/**
 * Represents an IDL fixed-width Int(n)
 */
export class FixedIntClass extends PrimitiveType<BigNumber | number> {
  constructor(private _bits: number) {
    super();
  }

  public covariant(x: any): x is BigNumber {
    const min = new BigNumber(2).pow(this._bits - 1).negated();
    const max = new BigNumber(2).pow(this._bits - 1).minus(1);
    if (x instanceof BigNumber && x.isInteger()) {
      return x.gte(min) && x.lte(max);
    } else if (Number.isInteger(x)) {
      const v = new BigNumber(x);
      return v.gte(min) && v.lte(max);
    } else {
      return false;
    }
  }

  public encodeValue(x: BigNumber | number) {
    return writeIntLE(x, this._bits / 8);
  }

  public encodeType() {
    const offset = Math.log2(this._bits) - 3;
    return slebEncode(-9 - offset);
  }

  public decodeValue(b: Pipe) {
    const num = readIntLE(b, this._bits / 8);
    if (this._bits <= 32) {
      return num.toNumber();
    } else {
      return num;
    }
  }

  get name() {
    return `int${this._bits}`;
  }

  public valueToString(x: BigNumber | number) {
    return x.toString();
  }
}

/**
 * Represents an IDL fixed-width Nat(n)
 */
export class FixedNatClass extends PrimitiveType<BigNumber | number> {
  constructor(private _bits: number) {
    super();
  }

  public covariant(x: any): x is BigNumber {
    const max = new BigNumber(2).pow(this._bits);
    if (x instanceof BigNumber && x.isInteger() && !x.isNegative()) {
      return x.lt(max);
    } else if (Number.isInteger(x) && x >= 0) {
      const v = new BigNumber(x);
      return v.lt(max);
    } else {
      return false;
    }
  }

  public encodeValue(x: BigNumber | number) {
    return writeUIntLE(x, this._bits / 8);
  }

  public encodeType() {
    const offset = Math.log2(this._bits) - 3;
    return slebEncode(-5 - offset);
  }

  public decodeValue(b: Pipe) {
    const num = readUIntLE(b, this._bits / 8);
    if (this._bits <= 32) {
      return num.toNumber();
    } else {
      return num;
    }
  }

  get name() {
    return `nat${this._bits}`;
  }

  public valueToString(x: BigNumber | number) {
    return x.toString();
  }
}

/**
 * Represents an IDL Array
 * @param {Type} t
 */
class VecClass<T> extends ConstructType<T[]> {
  constructor(protected _type: Type<T>) {
    super();
  }

  public covariant(x: any): x is T[] {
    return Array.isArray(x) && x.every(v => this._type.covariant(v));
  }

  public encodeValue(x: T[]) {
    const len = lebEncode(x.length);
    return Buffer.concat([len, ...x.map(d => this._type.encodeValue(d))]);
  }

  public _buildTypeTableImpl(typeTable: TypeTable) {
    this._type.buildTypeTable(typeTable);

    const opCode = slebEncode(IDLTypeIds.Vector);
    const buffer = this._type.encodeType(typeTable);
    typeTable.add(this, Buffer.concat([opCode, buffer]));
  }

  public decodeValue(b: Pipe): any[] {
    const len = lebDecode(b).toNumber();
    const rets: any[] = [];
    for (let i = 0; i < len; i++) {
      rets.push(this._type.decodeValue(b));
    }
    return rets;
  }

  get name() {
    return `vec ${this._type.name}`;
  }

  public valueToString(x: T[]) {
    const elements = x.map(e => this._type.valueToString(e));
    return 'vec {' + elements.join('; ') + '}';
  }
}

/**
 * Represents an IDL Option
 * @param {Type} t
 */
class OptClass<T> extends ConstructType<T | null> {
  constructor(protected _type: Type<T>) {
    super();
  }

  public covariant(x: any): x is T | null {
    return x === null || this._type.covariant(x);
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

    const opCode = slebEncode(IDLTypeIds.Opt);
    const buffer = this._type.encodeType(typeTable);
    typeTable.add(this, Buffer.concat([opCode, buffer]));
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
    return `opt ${this._type.name}`;
  }

  public valueToString(x: T | null) {
    if (x === null) {
      return 'null';
    } else {
      return `opt ${this._type.valueToString(x)}`;
    }
  }
}

/**
 * Represents an IDL Record
 * @param {Object} [fields] - mapping of function name to Type
 */
class RecordClass extends ConstructType<Record<string, any>> {
  protected readonly _fields: Array<[string, Type]>;

  constructor(fields: Record<string, Type> = {}) {
    super();
    this._fields = Object.entries(fields).sort((a, b) => idlLabelToId(a[0]) - idlLabelToId(b[0]));
  }

  public covariant(x: any): x is Record<string, any> {
    return (
      typeof x === 'object' &&
      this._fields.every(([k, t]) => {
        if (!x.hasOwnProperty(k)) {
          throw new Error(`Record is missing key "${k}".`);
        }
        return t.covariant(x[k]);
      })
    );
  }

  public encodeValue(x: Record<string, any>) {
    const values = this._fields.map(([key]) => x[key]);
    const bufs = zipWith(this._fields, values, ([, c], d) => c.encodeValue(d));
    return Buffer.concat(bufs);
  }

  public _buildTypeTableImpl(T: TypeTable) {
    this._fields.forEach(([_, value]) => value.buildTypeTable(T));
    const opCode = slebEncode(IDLTypeIds.Record);
    const len = lebEncode(this._fields.length);
    const fields = this._fields.map(([key, value]) =>
      Buffer.concat([lebEncode(idlLabelToId(key)), value.encodeType(T)]),
    );

    T.add(this, Buffer.concat([opCode, len, Buffer.concat(fields)]));
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
    return `record {${fields.join('; ')}}`;
  }

  public valueToString(x: Record<string, any>) {
    const values = this._fields.map(([key]) => x[key]);
    const fields = zipWith(this._fields, values, ([k, c], d) => k + '=' + c.valueToString(d));
    return `record {${fields.join('; ')}}`;
  }
}

/**
 * Represents Tuple, a syntactic sugar for Record.
 * @param {Type} components
 */
class TupleClass<T extends any[]> extends RecordClass {
  protected readonly _components: Type[];

  constructor(_components: Type[]) {
    const x: Record<string, any> = {};
    _components.forEach((e, i) => (x['_' + i + '_'] = e));
    super(x);
    this._components = _components;
  }

  public covariant(x: any): x is T {
    // `>=` because tuples can be covariant when encoded.
    return (
      Array.isArray(x) &&
      x.length >= this._fields.length &&
      this._components.every((t, i) => t.covariant(x[i]))
    );
  }

  public encodeValue(x: any[]) {
    const bufs = zipWith(this._components, x, (c, d) => c.encodeValue(d));
    return Buffer.concat(bufs);
  }

  public decodeValue(b: Pipe): T {
    return this._components.map(c => c.decodeValue(b)) as T;
  }
}

/**
 * Represents an IDL Variant
 * @param {Object} [fields] - mapping of function name to Type
 */
class VariantClass extends ConstructType<Record<string, any>> {
  private readonly _fields: Array<[string, Type]>;

  constructor(fields: Record<string, Type> = {}) {
    super();
    this._fields = Object.entries(fields).sort((a, b) => idlLabelToId(a[0]) - idlLabelToId(b[0]));
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
        const idx = lebEncode(i);
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
    const opCode = slebEncode(IDLTypeIds.Variant);
    const len = lebEncode(this._fields.length);
    const fields = this._fields.map(([key, value]) =>
      Buffer.concat([lebEncode(idlLabelToId(key)), value.encodeType(typeTable)]),
    );
    typeTable.add(this, Buffer.concat([opCode, len, ...fields]));
  }

  public decodeValue(b: Pipe) {
    const idx = lebDecode(b).toNumber();
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
    return `variant {${fields.join('; ')}}`;
  }

  public valueToString(x: Record<string, any>) {
    for (const [name, type] of this._fields) {
      if (x.hasOwnProperty(name)) {
        const value = type.valueToString(x[name]);
        return `variant {${name}=${value}}`;
      }
    }
    throw Error('Variant has no data: ' + x);
  }
}

/**
 * Represents a reference to an IDL type, used for defining recursive data
 * types.
 */
class RecClass<T = any> extends ConstructType<T> {
  private static _counter = 0;
  private _id = RecClass._counter++;
  private _type: ConstructType<T> | undefined = undefined;

  public fill(t: ConstructType<T>) {
    this._type = t;
  }

  public getType() {
    return this._type;
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

  public decodeValue(b: Pipe) {
    if (!this._type) {
      throw Error('Recursive type uninitialized.');
    }
    return this._type.decodeValue(b);
  }

  get name() {
    return `rec_${this._id}`;
  }

  public display() {
    if (!this._type) {
      throw Error('Recursive type uninitialized.');
    }
    return `&mu;${this.name}.${this._type.name}`;
  }

  public valueToString(x: T) {
    if (!this._type) {
      throw Error('Recursive type uninitialized.');
    }
    return this._type.valueToString(x);
  }
}

/**
 * Represents an async function which can return data.
 * @param argTypes Argument types.
 * @param retTypes Return types.
 * @param annotations Function annotations.
 */
export class FuncClass {
  constructor(
    public argTypes: Type[] = [],
    public retTypes: Type[] = [],
    public annotations: string[] = [],
  ) {}
  public display(): string {
    const args = this.argTypes.map(arg => arg.display()).join(', ');
    const rets = this.retTypes.map(arg => arg.display()).join(', ');
    const annon = ' ' + this.annotations.join(' ');
    return `(${args}) &rarr; (${rets})${annon}`;
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
  const len = lebEncode(args.length);
  const typs = Buffer.concat(argTypes.map(t => t.encodeType(typeTable)));
  const vals = Buffer.concat(
    zipWith(argTypes, args, (t, x) => {
      if (!t.covariant(x)) {
        throw new Error(`Invalid ${t.display()} argument: "${JSON.stringify(x)}"`);
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
    const len = lebDecode(pipe).toNumber();

    for (let i = 0; i < len; i++) {
      const ty = slebDecode(pipe).toNumber();
      switch (ty) {
        case IDLTypeIds.Opt:
          slebDecode(pipe);
          break;
        case IDLTypeIds.Vector:
          slebDecode(pipe);
          break;
        case IDLTypeIds.Record: {
          // record/tuple
          let objectLength = lebDecode(pipe).toNumber();
          while (objectLength--) {
            lebDecode(pipe);
            slebDecode(pipe);
          }
          break;
        }
        case IDLTypeIds.Variant: {
          // variant
          let variantLength = lebDecode(pipe).toNumber();
          while (variantLength--) {
            lebDecode(pipe);
            slebDecode(pipe);
          }
          break;
        }
        default:
          throw new Error('Illegal op_code: ' + ty);
      }
    }

    const length = lebDecode(pipe).toNumber();
    for (let i = 0; i < length; i++) {
      slebDecode(pipe);
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
export const Empty = new EmptyClass();
export const Bool = new BoolClass();
export const Unit = new UnitClass();
export const Text = new TextClass();
export const Int = new IntClass();
export const Nat = new NatClass();

export const Int8 = new FixedIntClass(8);
export const Int16 = new FixedIntClass(16);
export const Int32 = new FixedIntClass(32);
export const Int64 = new FixedIntClass(64);

export const Nat8 = new FixedNatClass(8);
export const Nat16 = new FixedNatClass(16);
export const Nat32 = new FixedNatClass(32);
export const Nat64 = new FixedNatClass(64);

export function Tuple<T extends any[]>(...types: T): TupleClass<T> {
  return new TupleClass(types);
}
export function Vec<T>(t: Type<T>): VecClass<T> {
  return new VecClass(t);
}
export function Opt<T>(t: Type<T>): OptClass<T> {
  return new OptClass(t);
}

export function Record(t: Record<string, Type>): RecordClass {
  return new RecordClass(t);
}
export function Variant(fields: Record<string, Type>) {
  return new VariantClass(fields);
}
export function Rec() {
  return new RecClass();
}

export function Func(args: Type[], ret: Type[], annotations: string[] = []) {
  return new FuncClass(args, ret, annotations);
}
