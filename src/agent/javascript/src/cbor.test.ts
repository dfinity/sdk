import BigNumber from 'bignumber.js';
import { Buffer } from 'buffer/';
import { decode, encode } from './cbor';
import { Principal } from './principal';
import { BinaryBlob, blobToHex, blobFromHex, blobFromUint8Array } from './types';
import { getCrc32 } from './utils/getCrc';
var base32 = require('base32.js');

test('round trip', () => {
  interface Data {
    a: number;
    b: string;
    c: BinaryBlob;
    d: { four: string };
    e: Principal;
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
    e: Principal.fromText('ic:FfFfFfFfFfFfFfFfd7'),
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

test('my random test', () => {
  const hexWithChecksum = 'aaaaa-aa';
  console.log(hexWithChecksum);
  let hex = hexWithChecksum.toLowerCase().replace(/-/g, '');
  console.log(hex);
  var decoder = new base32.Decoder({ type: 'rfc4648', lc: false });
  let result = decoder.write(hex).finalize();
  console.log(result);
  let arr = new Uint8Array(result);
  console.log(arr);
});

test('my random test 2', () => {
  const canisterID = 'w7x7r-cok77-xa';
  console.log(canisterID);
  let canisterIdNoDash = canisterID.toLowerCase().replace(/-/g, '');

  var decoder = new base32.Decoder({ type: 'rfc4648', lc: false });
  let result = decoder.write(canisterIdNoDash).finalize();
  let arr = new Uint8Array(result);

  let arr2 = arr.slice(4, arr.length);
  let blob = blobFromUint8Array(arr2);

  let prin = Principal.fromBlob(blob);

  const hex1 = prin.toHex().toLowerCase();
  let blobofHex1 = blobFromHex(hex1);
  let arrayfromblobofHex1 = Uint8Array.from(blobofHex1);

  let arrb = new ArrayBuffer(4); // an Int32 takes 4 bytes
  let view = new DataView(arrb);
  view.setUint32(0, getCrc32(hex1), false); // byteOffset = 0; litteEndian = false

  const checksumuint8array = Uint8Array.from(Buffer.from(arrb));

  var array = new Uint8Array([...checksumuint8array, ...arrayfromblobofHex1]);

  var encoder = new base32.Encoder({ type: 'rfc4648', lc: false });
  let str = encoder.write(array).finalize().toLowerCase();
  const finalResult = str.match(/.{1,5}/g).join('-');
  console.log(finalResult);
  expect(finalResult).toBe(canisterID);
});

test('empty canister ID', () => {
  const input: { a: Principal } = {
    a: Principal.fromText('aaaaa-aa'),
  };

  const output = decode<typeof input>(encode(input));

  const inputA = input.a;
  const outputA = output.a;

  expect(buf2hex((outputA as any) as Uint8Array)).toBe(inputA.toHex());
  expect(Principal.fromBlob(outputA as any).toText()).toBe('aaaaa-aa');
});

function buf2hex(buffer: Uint8Array) {
  // Construct an array such that each number is translated to the
  // hexadecimal equivalent, ensure it is a string and padded then
  // join the elements.
  return Array.prototype.map.call(buffer, x => ('00' + x.toString(16)).slice(-2)).join('');
}
