// tslint:disable
import * as IDL from './IDL';
import { Buffer } from 'buffer';

function testEncode(typ: IDL.Type, val: any, hex: string, _str: string) {
  expect(IDL.encode([typ], [val])).toEqual(Buffer.from(hex, 'hex'));
}

function testDecode(typ: IDL.Type, val: any, hex: string, _str: string) {
  expect(IDL.decode([typ], Buffer.from(hex, 'hex'))[0]).toEqual(val);
}

function test_(typ: IDL.Type, val: any, hex: string, str: string) {
  testEncode(typ, val, hex, str);
  testDecode(typ, val, hex, str);
}

function test_args(typs: IDL.Type[], vals: any[], hex: string, _str: string) {
  expect(IDL.encode(typs, vals)).toEqual(Buffer.from(hex, 'hex'));
  expect(IDL.decode(typs, Buffer.from(hex, 'hex'))).toEqual(vals);
}

test('IDL hash', () => {
  function testHash(str: string, hash: number) {
    expect(IDL.hash(str)).toBe(hash);
  }

  testHash('', 0);
  testHash('id', 23515);
  testHash('description', 1595738364);
  testHash('short_name', 3261810734);
  testHash('Hi ☃', 1419229646);
});

test('IDL encoding', () => {
  // Wrong magic number
  expect(() => IDL.decode([IDL.Nat], Buffer.from('2a'))).toThrow(
    /Message length smaller than magic number/,
  );
  expect(() => IDL.decode([IDL.Nat], Buffer.from('4449444d2a'))).toThrow(/Wrong magic number:/);

  // None
  expect(() => IDL.encode([IDL.None], [undefined])).toThrow(/Invalid None argument:/);
  expect(() => IDL.decode([IDL.None], Buffer.from('DIDL'))).toThrow(
    /None cannot appear as an output/,
  );

  // Unit
  test_(IDL.Unit, null, '4449444c00017f', 'Unit value');

  // Text
  test_(IDL.Text, 'Hi ☃\n', '4449444c00017107486920e298830a', 'Text with unicode');
  test_(
    IDL.Opt(IDL.Text),
    'Hi ☃\n',
    '4449444c016e7101000107486920e298830a',
    'Nested text with unicode',
  );
  expect(() => IDL.encode([IDL.Text], [0])).toThrow(/Invalid Text argument/);
  expect(() => IDL.encode([IDL.Text], [null])).toThrow(/Invalid Text argument/);

  // Int
  test_(IDL.Int, 42, '4449444c00017c2a', 'Int');
  test_(IDL.Int, 1234567890, '4449444c00017cd285d8cc04', 'Positive Int');
  test_(IDL.Int, -1234567890, '4449444c00017caefaa7b37b', 'Negative Int');

  // Nat
  test_(IDL.Nat, 42, '4449444c00017d2a', 'Nat');
  test_(IDL.Nat, 1234567890, '4449444c00017dd285d8cc04', 'Positive Nat');
  expect(() => IDL.encode([IDL.Nat], [-1])).toThrow(/Invalid Nat argument/);

  // Fixed-width number
  test_(IDL.Int8, 42, '4449444c0001772a', 'Int8');
  test_(IDL.Int32, 42, '4449444c0001750000002a', 'Int32');
  expect(() => IDL.encode([IDL.Int8], [256])).toThrow(/Invalid Int8 argument/);

  // Tuple
  test_(
    IDL.Tuple(IDL.Int, IDL.Text),
    [42, '💩'],
    '4449444c016c02007c017101002a04f09f92a9',
    'Pairs',
  );
  expect(() => IDL.encode([IDL.Tuple(IDL.Int, IDL.Text)], [[0]])).toThrow(
    /Invalid Tuple\(Int,Text\) argument/,
  );

  // Array
  test_(IDL.Arr(IDL.Int), [0, 1, 2, 3], '4449444c016d7c01000400010203', 'Array of Ints');
  expect(() => IDL.encode([IDL.Arr(IDL.Int)], [0])).toThrow(/Invalid Arr\(Int\) argument/);
  expect(() => IDL.encode([IDL.Arr(IDL.Int)], [['fail']])).toThrow(/Invalid Arr\(Int\) argument/);

  // Array of Tuple
  test_(
    IDL.Arr(IDL.Tuple(IDL.Int, IDL.Text)),
    [[42, 'text']],
    '4449444c026c02007c01716d000101012a0474657874',
    'Arr of Tuple',
  );

  // Nested Tuples
  test_(
    IDL.Tuple(IDL.Tuple(IDL.Tuple(IDL.Tuple(IDL.Unit)))),
    [[[[null]]]],
    '4449444c046c01007f6c0100006c0100016c0100020103',
    'Nested Tuples',
  );

  // Object
  test_(IDL.Obj({}), {}, '4449444c016c000100', 'Empty object');
  expect(() => IDL.encode([IDL.Obj({ a: IDL.Text })], [{ b: 'b' }])).toThrow(/Obj is missing key/);

  // Test that additional keys are ignored
  testEncode(
    IDL.Obj({ foo: IDL.Text, bar: IDL.Int }),
    { foo: '💩', bar: 42, baz: 0 },
    '4449444c016c02d3e3aa027c868eb7027101002a04f09f92a9',
    'Object',
  );
  testEncode(
    IDL.Obj({ foo: IDL.Text, bar: IDL.Int }),
    { foo: '💩', bar: 42 },
    '4449444c016c02d3e3aa027c868eb7027101002a04f09f92a9',
    'Object',
  );

  // Bool
  test_(IDL.Bool, true, '4449444c00017e01', 'true');
  test_(IDL.Bool, false, '4449444c00017e00', 'false');
  expect(() => IDL.encode([IDL.Bool], [0])).toThrow(/Invalid Bool argument/);
  expect(() => IDL.encode([IDL.Bool], ['false'])).toThrow(/Invalid Bool argument/);

  // Variants
  const Result = IDL.Variant({ ok: IDL.Text, err: IDL.Text });
  test_(Result, { ok: 'good' }, '4449444c016b029cc20171e58eb4027101000004676f6f64', 'Result ok');
  test_(Result, { err: 'uhoh' }, '4449444c016b029cc20171e58eb402710100010475686f68', 'Result err');
  expect(() => IDL.encode([Result], [{}])).toThrow(/Invalid Variant\(ok:Text,err:Text\) argument/);
  expect(() => IDL.encode([Result], [{ ok: 'ok', err: 'err' }])).toThrow(
    /Invalid Variant\(ok:Text,err:Text\) argument/,
  );

  // Test that nullary constructors work as expected
  test_(
    IDL.Variant({ foo: IDL.Unit }),
    { foo: null },
    '4449444c016b01868eb7027f010000',
    'Nullary constructor in variant',
  );

  // Test that None within variants works as expected
  test_(
    IDL.Variant({ ok: IDL.Text, err: IDL.None }),
    { ok: 'good' },
    '4449444c016b029cc20171e58eb4026f01000004676f6f64',
    'None within variants',
  );
  expect(() =>
    IDL.encode([IDL.Variant({ ok: IDL.Text, err: IDL.None })], [{ err: 'uhoh' }]),
  ).toThrow(/Invalid Variant\(ok:Text,err:None\) argument:/);

  // Test for option
  test_(IDL.Opt(IDL.Nat), null, '4449444c016e7d010000', 'Null option');
  test_(IDL.Opt(IDL.Nat), 1, '4449444c016e7d01000101', 'Non-null option');

  // Type description sharing
  test_(
    IDL.Tuple(IDL.Arr(IDL.Int), IDL.Arr(IDL.Nat), IDL.Arr(IDL.Int), IDL.Arr(IDL.Nat)),
    [[], [], [], []],
    '4449444c036d7c6d7d6c040000010102000301010200000000',
    'Type sharing',
  );

  // Test for recursive types
  const List = IDL.Rec();
  expect(() => IDL.encode([List], [null])).toThrow(/Recursive type uninitialized/);
  List.fill(IDL.Opt(IDL.Obj({ head: IDL.Int, tail: List })));
  test_(List, null, '4449444c026e016c02a0d2aca8047c90eddae70400010000', 'Empty list');
  test_(
    List,
    { head: 1, tail: { head: 2, tail: null } },
    '4449444c026e016c02a0d2aca8047c90eddae7040001000101010200',
    'List',
  );

  // Mutual recursion
  const List1 = IDL.Rec();
  const List2 = IDL.Rec();
  List1.fill(IDL.Opt(List2));
  List2.fill(IDL.Obj({ head: IDL.Int, tail: List1 }));
  test_(List1, null, '4449444c026e016c02a0d2aca8047c90eddae70400010000', 'Empty list');
  test_(
    List1,
    { head: 1, tail: { head: 2, tail: null } },
    '4449444c026e016c02a0d2aca8047c90eddae7040001000101010200',
    'List',
  );

  // Test for multiple arguments
  test_args(
    [IDL.Nat, IDL.Opt(IDL.Text), Result],
    [42, 'test', { ok: 'good' }],
    '4449444c026e716b029cc20171e58eb40271037d00012a0104746573740004676f6f64',
    'Multiple arguments',
  );
  test_args([], [], '4449444c0000', 'empty args');
});
