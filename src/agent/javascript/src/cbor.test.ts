import BigNumber from 'bignumber.js';
import { Buffer } from 'buffer/';
import { CanisterId } from './canisterId';
import { decode, encode } from './cbor';
import { BinaryBlob, blobToHex } from './types';

test('round trip', () => {
  interface Data {
    a: number;
    b: string;
    c: BinaryBlob;
    d: { four: string };
    e: CanisterId;
    f: BinaryBlob;
    g: BigNumber;
  }

  // FIXME: since we have limited control over CBOR decoding, we are relying on
  // BigNumber types actually containing big numbers, since small numbers are
  // represented as numbers and big numbers are represented as strings.
  const input: Data = {
    a: 1,
    b: 'two',
    c: Buffer.from([3]) as BinaryBlob,
    d: { four: 'four' },
    e: CanisterId.fromText('ic:FfFfFfFfFfFfFfFfd7'),
    f: Buffer.from([]) as BinaryBlob,
    g: new BigNumber('0xffffffffffffffff'),
  };

  const output = decode<Data>(encode(input));

  // Some values don't decode exactly to the value that was encoded,
  // but their hexadecimal representions are the same.
  const { c: inputC, e: inputE, f: inputF, ...inputRest } = input;

  const { c: outputC, e: outputE, f: outputF, ...outputRest } = output;

  expect(blobToHex(outputC)).toBe(blobToHex(inputC));
  expect(buf2hex((outputE as any) as Uint8Array).toUpperCase()).toBe(inputE.toHex());
  expect(outputRest).toEqual(inputRest);
});

test('empty canister ID', () => {
  const input: { a: CanisterId } = {
    a: CanisterId.fromText('ic:00'),
  };

  const output = decode<typeof input>(encode(input));

  const inputA = input.a;
  const outputA = output.a;

  expect(buf2hex((outputA as any) as Uint8Array)).toBe(inputA.toHex());
  expect(CanisterId.fromBlob(outputA as any).toText()).toBe('ic:00');
});

function buf2hex(buffer: Uint8Array) {
  // Construct an array such that each number is translated to the
  // hexadecimal equivalent, ensure it is a string and padded then
  // join the elements.
  return Array.prototype.map.call(buffer, x => ('00' + x.toString(16)).slice(-2)).join('');
}
