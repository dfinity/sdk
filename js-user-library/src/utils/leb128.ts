// tslint:disable:no-bitwise
import BigNumber from 'bignumber.js';
import Pipe = require('buffer-pipe');
import { Buffer } from 'buffer/';

export function lebEncode(value: number | BigNumber): Buffer {
  if (typeof value === 'number') {
    value = new BigNumber(value);
  }
  value = value.integerValue();
  if (value.lt(0)) {
    throw new Error('Cannot leb encode negative values.');
  }
  if (value.eq(0)) {
    // Clamp to 0.
    return Buffer.from([0]);
  }

  const pipe = new Pipe();
  while (value.gt(0)) {
    const i = value.mod(0x80).toNumber();
    value = value.idiv(0x80);

    if (value.gt(0)) {
      pipe.write([i | 0x80]);
    } else {
      pipe.write([i]);
    }
  }

  return pipe.buffer;
}

export function lebDecode(pipe: Pipe): BigNumber {
  let shift = 0;
  let value = new BigNumber(0);
  let byte;

  do {
    byte = pipe.read(1)[0];
    value = value.plus(new BigNumber(byte & 0x7f).multipliedBy(new BigNumber(2).pow(shift)));
    shift += 7;
  } while (byte >= 0x80);

  return value;
}

export function slebEncode(value: BigNumber | number): Buffer {
  if (typeof value === 'number') {
    value = new BigNumber(value);
  }

  if (value.gte(0)) {
    return lebEncode(value);
  }

  value = value
    .abs()
    .integerValue()
    .minus(1);

  // We need to special case 0, as it would return an empty buffer. Since
  // we removed 1 above, this is really -1.
  if (value.eq(0)) {
    return Buffer.from([0x7f]);
  }

  const pipe = new Pipe();
  while (value.gt(0)) {
    // We swap the bits here again, and remove 1 to do two's complement.
    const i = 0x80 - value.mod(0x80).toNumber() - 1;
    value = value.idiv(0x80);

    if (value.gt(0)) {
      pipe.write([i | 0x80]);
    } else {
      pipe.write([i]);
    }
  }

  return pipe.buffer;
}

export function slebDecode(pipe: Pipe): BigNumber {
  // Get the size of the buffer, then cut a buffer of that size.
  const pipeView = new Uint8Array(pipe.buffer);
  let len = 0;
  for (; len < pipeView.byteLength; len++) {
    if (pipeView[len] < 0x80) {
      // If it's a positive number, we reuse lebDecode.
      if ((pipeView[len] & 0x40) === 0) {
        return lebDecode(pipe);
      }
      break;
    }
  }

  const bytes = new Uint8Array(pipe.read(len + 1));
  let value = new BigNumber(0);
  for (let i = bytes.byteLength - 1; i >= 0; i--) {
    value = value.times(0x80).plus(0x80 - (bytes[i] & 0x7f) - 1);
  }
  return value.negated().minus(1);
}

export function writeUIntLE(value: BigNumber | number, byteLength: number): Buffer {
  if ((value instanceof BigNumber && value.isNegative()) || value < 0) {
    throw new Error('Cannot write negative values.');
  }
  return writeIntLE(value, byteLength);
}

export function writeIntLE(value: BigNumber | number, byteLength: number): Buffer {
  if (typeof value === 'number') {
    value = new BigNumber(value);
  }
  value = value.integerValue();
  const pipe = new Pipe();
  let i = 0;
  let mul = new BigNumber(256);
  let sub = 0;
  let byte = value.mod(mul).toNumber();
  pipe.write([byte]);
  while (++i < byteLength) {
    if (value.lt(0) && sub === 0 && byte !== 0) {
      sub = 1;
    }
    byte = value
      .idiv(mul)
      .minus(sub)
      .mod(256)
      .toNumber();
    pipe.write([byte]);
    mul = mul.times(256);
  }
  return pipe.buffer;
}

export function readUIntLE(pipe: Pipe, byteLength: number): BigNumber {
  let val = new BigNumber(pipe.read(1)[0]);
  let mul = new BigNumber(1);
  let i = 0;
  while (++i < byteLength) {
    mul = mul.times(256);
    const byte = pipe.read(1)[0];
    val = val.plus(mul.times(byte));
  }
  return val;
}

export function readIntLE(pipe: Pipe, byteLength: number): BigNumber {
  let val = readUIntLE(pipe, byteLength);
  const mul = new BigNumber(2).pow(8 * (byteLength - 1) + 7);
  if (val.gte(mul)) {
    val = val.minus(mul.times(2));
  }
  return val;
}
