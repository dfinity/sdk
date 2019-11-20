const { TextEncoder } = require('text-encoding')

const sleb = require('leb128/signed')
const leb = require('leb128/unsigned')
const Pipe = require('buffer-pipe')

const type = o => Object.prototype.toString.call(o).slice(8, -1)

/*
  This module provides a combinator library to create serializers/deserializers
  between JavaScript values and IDL used by canisters on the Internet Computer,
  as documented at https://github.com/dfinity-lab/actorscript/blob/128e37bf6800125056269454a21acd8f2c70b226/design/IDL.md
*/

/*
  The function `encode` takes a JavaScript value that can be represented as
  such, and turns it into a Buffer.
*/

const zipWith = (xs, ys, f) => xs.map((x, i) => f(x, ys[i]))

const idlHash = s => {
  if (!(typeof s === 'string')) {
    throw Error('idlHash: Argument not a string: ' + s)
  }
  const utf8encoder = new TextEncoder();
  const array = utf8encoder.encode(s);
  var h = 0;
  array.forEach((c, i, array) => { h *= 223; h += c; h %= 2**32; })
  return h
}

const magicNumber = 'DIDL'

class TypeTable {
  constructor () {
    this.__typs = []  // array(buffer)
    this.__idx = {}   // map(type_string, index)
  }

  hasType (obj) {
    return this.__idx.hasOwnProperty(obj)
  }

  addType (obj, buf) {
    if (this.__idx.hasOwnProperty(obj)) {
      throw new Error('duplicate type name: ' + obj)
    }
    const idx = this.__typs.length
    this.__idx[obj] = idx
    this.__typs.push(buf)
  }

  mergeType (obj, knot) {
    if (!this.__idx.hasOwnProperty(obj)) {
      throw new Error('Missing type index for ' + obj)
    }
    if (!this.__idx.hasOwnProperty(knot)) {
      throw new Error('Missing type index for ' + knot)
    }
    const idx = this.__idx[obj]
    const knot_idx = this.__idx[knot]
    this.__typs[idx] = this.__typs[knot_idx]
    this.__typs.splice(knot_idx, 1)
    delete this.__idx[knot]
  }

  getTypeIdx (obj) {
    if (!this.__idx.hasOwnProperty(obj)) {
      throw new Error('Missing type index for ' + obj)
    }
    return sleb.encode(this.__idx[obj])
  }

  encodeTable () {
    const len = leb.encode(this.__typs.length)
    const buf = Buffer.concat(this.__typs)
    return Buffer.concat([len, buf])
  }
}

/**
 * Represents an IDL type
 */
class Type {
  /* Memoized DFS for storing type description into TypeTable  */
  buildType (T) {
    if (!T.hasType(this.toString())) return this.buildTypeGo(T)
  }

  /* Implement T in the IDL spec,
   * only needed for non-primitive types */
  buildTypeGo (T) { }

  encodeGo (x) {
    throw new Error('You have to implement the method encodeGo!')
  }

  /* Implement I in the IDL spec */
  encodeTypeGo (T) {
    throw new Error('You have to implement the method encodeTypeGo!')
  }

  decodeGo (x) {
    throw new Error('You have to implement the method decodeGo!')
  }

  toString () {
    return this.constructor.name
  }
}

/**
 * Represents an IDL None, a type which has no inhabitants.
 * Since no values exist for this type, it cannot be serialised or deserialised.
 * Result types like `Result<Text, None>` should always succeed.
 */
class None extends Type {
  encodeGo (x) {
    throw new Error('None cannot appear as a function argument')
  }

  encodeTypeGo (T) {
    return sleb.encode(-17)
  }

  decodeGo (b) {
    throw new Error('None cannot appear as an output')
  }
}

/**
 * Represents an IDL Bool
 */
class Bool extends Type {
  encodeGo (x) {
    if (typeof x !== 'boolean') {
      throw Error('Invalid Bool argument: ' + x)
    }
    const buf = Buffer.alloc(1)
    buf.writeInt8(x ? 1 : 0, 0)
    return buf
  }

  encodeTypeGo (T) {
    return sleb.encode(-2)
  }

  decodeGo (b) {
    const x = b.read(1).toString('hex')
    return x === "01"
  }
}

/**
 * Represents an IDL Unit
 */
class Unit extends Type {
  encodeGo (x) {
    return Buffer.alloc(0)
  }

  encodeTypeGo (T) {
    return sleb.encode(-1)
  }

  decodeGo (b) {
    return null
  }
}

/**
 * Represents an IDL Text
 */
class Text extends Type {
  encodeGo (x) {
    if (typeof x !== 'string') {
      throw Error('Invalid Text argument: ' + x)
    }

    const buf = Buffer.from(x, 'utf8')
    const len = leb.encode(buf.length)
    return Buffer.concat([len, buf])
  }

  encodeTypeGo (T) {
    return sleb.encode(-15)
  }

  decodeGo (b) {
    const len = leb.readBn(b).toNumber()
    const x = b.read(len).toString('utf8')
    return x
  }
}

/**
 * Represents an IDL Int
 */
class Int extends Type {
  encodeGo (x) {
    if (!Number.isInteger(x)) {
      throw Error('Invalid Int argument: ' + x)
    }
    return sleb.encode(x)
  }

  encodeTypeGo (T) {
    return sleb.encode(-4)
  }

  decodeGo (b) {
    return sleb.readBn(b).toNumber()
  }
}

/**
 * Represents an IDL Nat
 */
class Nat extends Type {
  encodeGo (x) {
    if (!Number.isInteger(x) || x < 0) {
      throw Error('Invalid Nat argument: ' + x)
    }
    return leb.encode(x)
  }

  encodeTypeGo (T) {
    return sleb.encode(-3)
  }

  decodeGo (b) {
    return leb.readBn(b).toNumber()
  }
}

/**
 * Represents an IDL Tuple
 * @param {Type} components
*/
class Tuple extends Type {
  constructor (...components) {
    super()
    this.__components = components
  }

  encodeGo (x) {
    if (x.length !== this.__components.length) { throw Error(`Tuple argument has wrong length, want: (${this.__components})`) };
    const bufs = zipWith(this.__components, x, (c, d) => c.encodeGo(d))
    return Buffer.concat(bufs)
  }

  buildTypeGo (T) {
    this.__components.forEach (x => x.buildType(T))
    const op_code = sleb.encode(-20)
    const len = leb.encode(this.__components.length)
    const buf = Buffer.concat(this.__components.map (
      (x, i) => Buffer.concat([leb.encode(i) , x.encodeTypeGo(T)])))
    T.addType(this, Buffer.concat([op_code, len, buf]))
  }

  encodeTypeGo (T) {
    return T.getTypeIdx(this)
  }

  decodeGo (b) {
    return this.__components.map(c => c.decodeGo(b))
  }

  toString () {
    const name = super.toString()
    return `${name}(${this.__components})`
  }
}

/**
 * Represents an IDL Array
 * @param {Type} t
 */
class Arr extends Type {
  constructor (t) {
    super()
    this.__typ = t
  }

  encodeGo (x) {
    if (!Array.isArray(x)) {
      throw Error('Invalid Arr argument: ' + x)
    }

    const len = leb.encode(x.length)
    const xs = x.map(d => this.__typ.encodeGo(d))
    const bufs = [len].concat(xs)
    return Buffer.concat(bufs)
  }

  buildTypeGo (T) {
    this.__typ.buildType(T)
    const op_code = sleb.encode(-19)
    const t_buf = this.__typ.encodeTypeGo(T)
    T.addType(this, Buffer.concat([op_code, t_buf]))
  }

  encodeTypeGo (T) {
    return T.getTypeIdx(this)
  }

  decodeGo (b) {
    const len = leb.readBn(b).toNumber()
    const rets = []
    for (let i = 0; i < len; i++) {
      rets.push(this.__typ.decodeGo(b))
    }
    return rets
  }

  toString () {
    const name = super.toString()
    return `${name}(${this.__typ})`
  }
}

/**
 * Represents an IDL Option
 * @param {Type} t
 */
class Opt extends Type {
  constructor (t) {
    super()
    this.__typ = t
  }

  encodeGo (x) {
    if (x == null) {
      return Buffer.from([0])
    } else {
      return Buffer.concat([Buffer.from([1]), this.__typ.encodeGo(x)])
    }
  }

  buildTypeGo (T) {
    this.__typ.buildType(T)
    const op_code = sleb.encode(-18)
    const t_buf = this.__typ.encodeTypeGo(T)
    const buf = Buffer.concat([op_code, t_buf])
    T.addType(this, buf)
  }

  encodeTypeGo (T) {
    return T.getTypeIdx(this)
  }

  decodeGo (b) {
    const len = b.read(1).toString('hex')
    if (len === "00") {
      return null
    } else {
      return this.__typ.decodeGo(b)
    }
  }

  toString () {
    const name = super.toString()
    return `${name}(${this.__typ})`
  }
}

/**
 * Represents an IDL Object
 * @param {Object} [fields] - mapping of function name to Type
 */
class Obj extends Type {
  constructor (fields = {}) {
    super()
    const sortedFields = Object.entries(fields)
      .sort((a, b) => idlHash(a[0]) - idlHash(b[0]))
    this.__fields = sortedFields
  }

  encodeGo (x) {
    const values = this.__fields.map (
      ([key, value]) => {
        if (!x.hasOwnProperty(key)) {
          throw Error('Obj is missing key: ' + key)
        }
        return x[key]
      })
    const bufs = zipWith(this.__fields, values, ([_, c], d) => c.encodeGo(d))
    return Buffer.concat(bufs)
  }

  buildTypeGo (T) {
    this.__fields.forEach (([key, value]) => value.buildType(T))
    const op_code = sleb.encode(-20)
    const len = leb.encode(this.__fields.length)
    const fields = this.__fields.map (
      ([key, value]) => Buffer.concat(
        [leb.encode(idlHash(key)), value.encodeTypeGo(T)]))
    T.addType(this, Buffer.concat([op_code, len, Buffer.concat(fields)]))
  }

  encodeTypeGo (T) {
    return T.getTypeIdx(this)
  }

  decodeGo (b) {
    const x = {}
    this.__fields.forEach(([key, value]) => {
      x[key] = value.decodeGo(b)
    })
    return x
  }

  toString () {
    const name = super.toString()
    const fields = this.__fields.map (([key, value]) => key + ':' + value)
    return `${name}(${fields})`
  }
}

/**
 * Represents an IDL Variant
 * @param {Object} [fields] - mapping of function name to Type
 */
class Variant extends Type {
  constructor (fields = {}) {
    super()
    this.__fields = Object.entries(fields)
      .sort((a, b) => idlHash(a[0]) - idlHash(b[0]))
  }

  encodeGo (x) {
    let out
    for (var i = 0; i < this.__fields.length; i++) {
      const [k, v] = this.__fields[i]
      if (x.hasOwnProperty(k)) {
        if (out) {
          throw Error('Variant has extra key: ' + k)
        }
        const idx = leb.encode(i)
        const buf = v.encodeGo(x[k])
        out = Buffer.concat([idx, buf])
      }
    }
    if (!out) {
      throw Error('Variant has no data: ' + x)
    }
    return out
  }

  buildTypeGo (T) {
    this.__fields.forEach (([key, value]) => value.buildType(T))
    const op_code = sleb.encode(-21)
    const len = leb.encode(this.__fields.length)
    const fields = this.__fields.map (
      ([key, value]) => Buffer.concat(
        [leb.encode(idlHash(key)), value.encodeTypeGo(T)]))
    T.addType(this, Buffer.concat([op_code, len, Buffer.concat(fields)]))
  }

  encodeTypeGo (T) {
    return T.getTypeIdx(this)
  }

  decodeGo (b) {
    const idx = leb.readBn(b).toNumber()
    if (idx >= this.__fields.length) {
      throw Error('Invalid variant: ' + idx)
    }

    const value = this.__fields[idx][1].decodeGo(b)
    return {
      [this.__fields[idx][0]]: value
    }
  }

  toString () {
    const name = super.toString()
    const fields = this.__fields.map (([key, value]) => key + ':' + value)
    return `${name}(${fields})`
  }
}

/**
 * Represents a reference to an IDL type,
 * used for defining recursive data types.
 */
class Rec extends Type {
  constructor () {
    super()
    if (typeof Rec.counter === 'undefined') {
      Rec.counter = 0;
    }
    this.__id = Rec.counter++;
  }

  fill (t) {
    this.__typ = t
  }

  encodeGo (x) {
    if (!this.hasOwnProperty('__typ')) {
      throw Error('Recursive type uninitialized.')
    }
    return this.__typ.encodeGo(x)
  }

  buildTypeGo (T) {
    if (!this.hasOwnProperty('__typ')) {
      throw Error('Recursive type uninitialized.')
    }
    T.addType(this, Buffer.alloc(0))
    this.__typ.buildType(T)
    T.mergeType(this, this.__typ)
  }

  encodeTypeGo(T) {
    return T.getTypeIdx(this)
  }

  decodeGo (b) {
    if (!this.hasOwnProperty('__typ')) {
      throw Error('Recursive type uninitialized.')
    }
    return this.__typ.decodeGo(b)
  }

  toString () {
    const name = super.toString()
    return `${name}(${this.__id})`
  }
}

/**
 * Represents an async function which can return data
 * @param {Array<Type>} [argTypes] - argument types
 * @param {Array<Type>} [retTypes] - return types
 */
class Func {
  constructor (argTypes = [], retTypes = null) {
    if (argTypes instanceof Type) {
      argTypes = [ argTypes ]
    }
    if (retTypes instanceof Type) {
      retTypes = [ retTypes ]
    }
    if (!Array.isArray(argTypes)) {
      throw Error('First argument to Func must be an array of IDL argument types.')
    }
    if (retTypes && !Array.isArray(retTypes)) {
      throw Error('Second argument to Func must be an array of IDL argument types.')
    }
    this.argTypes = argTypes
    this.retTypes = retTypes
  }
}

/**
 * Encode a array of values
 * @returns {Buffer} serialised value
*/
function encode(argTypes, args) {
  if (args.length !== argTypes.length) {
    throw Error('Wrong number of message arguments')
  }
  const T = new TypeTable()
  argTypes.map(t => t.buildType(T))
  
  const magic = Buffer.from(magicNumber, 'utf8')
  const table = T.encodeTable()
  const len = leb.encode(args.length)
  const typs = Buffer.concat(argTypes.map(t => t.encodeTypeGo(T)))
  const vals = Buffer.concat(zipWith(argTypes, args, (t, x) => t.encodeGo(x)))
  return Buffer.concat([magic,table,len,typs,vals])
}

/**
 * Decode a binary value
 * @param {string|Buffer} x - hex-encoded string, or buffer
 * @returns {bool|string|number|Array|Object} value deserialised to JS type
*/
function decode(retTypes, bytes) {
  if (type(bytes) === 'Error') {
    throw bytes
  }
  const decodeType = b => {
    if (b.buffer.length < magicNumber.length) {
      throw new Error('Message length smaller than magic number')
    }
    const magic = b.read(magicNumber.length).toString()
    if (magic !== magicNumber) {
      throw new Error('Wrong magic number: ' + magic)
    }
    const len = leb.readBn(b).toNumber()
    for (var i = 0; i < len; i++) {
      const ty = sleb.readBn(b).toNumber()
      switch (ty) {
      case -18:   // opt
        sleb.readBn(b).toNumber()
        break
      case -19:   // vec
        sleb.readBn(b).toNumber()
        break
      case -20:   // record/tuple
        var obj_len = leb.readBn(b).toNumber()
        while (obj_len--) {
          leb.readBn(b).toNumber()
          sleb.readBn(b).toNumber()
        }
        break
      case -21:   // variant
        var var_len = leb.readBn(b).toNumber()
        while (var_len--) {
          leb.readBn(b).toNumber()
          sleb.readBn(b).toNumber()
        }
        break
      default:
        throw new Error('Illegal op_code: ' + ty)
      }
    }
    const ty_len = leb.readBn(b)
    for (var i = 0; i < ty_len; i++) {
      sleb.readBn(b).toNumber()
    }
  }
  const b = new Pipe(bytes)
  const _ = decodeType(b)
  const output = retTypes.map(t => t.decodeGo(b))
  if (b.buffer.length > 0) {
    throw new Error('decode: Left-over bytes')
  }
  return output
}

/**
 * A wrapper over a client and an IDL
 * @param {Object} [fields] - a map of function names to IDL function signatures
 */
class ActorInterface {
  constructor (fields) {
    this.__id = null
    this.__fields = fields
  }

  /**
   * Deploy a new actor
   * @param {Client} client
   * @param {Buffer} wasm - wasm Module
   * @param {string} actorName - local name to save actor as, which is _not_ sent to the network
   * @param {Buffer} [secretKey] - ed25519 secret key to sign Payment with
   * @param {Boolean} [proxy] - use the proxy actor
   * @returns {this} wrapper over the client and new actor
   */
  async new (client, wasm, actorName, secretKey, proxy = false) {
    const actorId = await client.newAndCall(actorName, wasm, 'start', [], secretKey, {}, proxy)
    return this.fromId(client, actorId, secretKey, proxy)
  }

  /**
   * Use an existing actor
   * @param {Client} client
   * @param {string} actorId - existing actor ID
   * @param {Buffer} [secretKey] - ed25519 secret key to sign Payment with
   * @param {Boolean} [proxy] - use the proxy instead of the specified actor
   * @returns {this} wrapper over the client and existing actor
   */
  fromId (client, actorId, secretKey, proxy = false) {
    const actor = {
      __id: actorId,
      batch: () => {
        this.__batch = true
        return async ps => {
          this.__batch = false
          const fs = await Promise.all(ps)
          const outs = await client.batch(fs.map(f => f.payment))
          return zipWith(fs, outs, (f, o) => f.cb(o))
        }
      }
    }

    for (const key in this.__fields) {
      if (this.__fields.hasOwnProperty(key)) {
        const msg = this.__fields[key]

        actor[key] = async (...args) => {
          const rawArgs = encode(msg.argTypes, args)
          if (this.__batch) {
            if (msg.retTypes) {
              return {
                payment: client.makeCallResult(actorId, key, rawArgs, secretKey, {}, proxy),
                cb: decodeResults
              }
            } else {
              return {
                payment: client.makeCall(actorId, key, rawArgs, secretKey, {}, proxy),
                cb: Function.prototype
              }
            }
          } else {
            if (msg.retTypes) {
              const rawOutputs = await client.callResult(actorId, key, rawArgs, secretKey, {}, proxy)
              return decode(msg.retTypes, rawOutputs)
            } else {
              return client.call(actorId, key, rawArgs, secretKey, {}, proxy)
            }
          }
        }
      }
    }
    return actor
  }

  /**
   * Deploy a new actor using a proxy
   * @param {Client} client
   * @param {Buffer} wasm - wasm Module
   * @param {Buffer} [secretKey] - ed25519 secret key to sign Payment with
   * @returns {this} wrapper over the client and new actor
   */
  async newProxied (client, wasm, secretKey) {
    return this.new(client, wasm, null, secretKey, true)
  }

  /**
   * Use the proxied actor
   * @param {Client} client
   * @param {Buffer} [secretKey] - ed25519 secret key to sign Payment with
   * @returns {this} wrapper over the client and existing actor
   */
  fromProxy (client, secretKey) {
    return this.fromId(client, null, secretKey, true)
  }
}

// Exports

module.exports = {
  None: new None(),
  Bool: new Bool(),
  Unit: new Unit(),
  Text: new Text(),
  Int: new Int(),
  Nat: new Nat(),
  Tuple: (...tys) => new Tuple(...tys),
  Arr: t => new Arr(t),
  Opt: t => new Opt(t),
  Obj: fs => new Obj(fs),
  Variant: fs => new Variant(fs),
  Rec : () => new Rec(),

  Func: (...args) => new Func(...args),
  ActorInterface,
  idlHash,
  encode,
  decode,
}
