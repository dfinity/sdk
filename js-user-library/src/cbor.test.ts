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
    e: CanisterId.fromHex('ffffffffffffffff'),
    f: Buffer.from([]) as BinaryBlob,
    g: new BigNumber('0xffffffffffffffff'),
  };

  const output = decode<Data>(encode(input)) as Data;

  // Some values don't decode exactly to the value that was encoded,
  // but their hexadecimal representions are the same.
  const { c: inputC, e: inputE, f: inputF, ...inputRest } = input;

  const { c: outputC, e: outputE, f: outputF, ...outputRest } = output;

  expect(blobToHex(outputC)).toBe(blobToHex(inputC));
  expect(((outputE as any) as BigNumber).toString(16)).toBe(inputE.toHex());
  expect(outputRest).toEqual(inputRest);
});
