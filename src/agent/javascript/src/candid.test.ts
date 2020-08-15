import { Buffer } from 'buffer/';
import * as IDL from './idl';

test('1:empty', () => {
  expect(() => IDL.decode([], Buffer.from('', 'hex'))).toThrow();
});
test('2:no magic bytes', () => {
  expect(() => IDL.decode([], Buffer.from('0000', 'hex'))).toThrow();
});
test('3:wrong magic bytes', () => {
  expect(() => IDL.decode([], Buffer.from('4441444c', 'hex'))).toThrow();
});
test('4:wrong magic bytes', () => {
  expect(() => IDL.decode([], Buffer.from('4441444c0000', 'hex'))).toThrow();
});
test('5:overlong typ table length', () => {
  expect(IDL.decode([], Buffer.from('4449444c800000', 'hex'))).toEqual(expect.anything());
});
test('6:overlong arg length', () => {
  expect(IDL.decode([], Buffer.from('4449444c008000', 'hex'))).toEqual(expect.anything());
});
test('7', () => {
  expect(IDL.decode([], Buffer.from('4449444c0000', 'hex'))).toEqual(expect.anything());
});
test('8:nullary: too long', () => {
  expect(() => IDL.decode([], Buffer.from('4449444c000000', 'hex'))).toThrow();
});
test('9:Additional parameters are ignored', () => {
  expect(IDL.decode([], Buffer.from('4449444c00017f', 'hex'))).toEqual(expect.anything());
});
test('10', () => {
  expect(IDL.decode([IDL.Null], Buffer.from('4449444c00017f', 'hex'))).toEqual(expect.anything());
});
test('11:null: too long', () => {
  expect(() => IDL.decode([IDL.Null], Buffer.from('4449444c00017f00', 'hex'))).toThrow();
});
test('12:null: missing', () => {
  expect(() => IDL.decode([IDL.Null], Buffer.from('4449444c0000', 'hex'))).toThrow();
});
test('13:bool: false', () => {
  expect(IDL.decode([IDL.Bool], Buffer.from('4449444c00017e00', 'hex'))).toEqual(expect.anything());
});
test('14:bool: true', () => {
  expect(IDL.decode([IDL.Bool], Buffer.from('4449444c00017e01', 'hex'))).toEqual(expect.anything());
});
test('15:bool: missing', () => {
  expect(() => IDL.decode([IDL.Bool], Buffer.from('4449444c00017e', 'hex'))).toThrow();
});
test('16:bool: out of range', () => {
  expect(() => IDL.decode([IDL.Bool], Buffer.from('4449444c00017e02', 'hex'))).toThrow();
});
test('17:bool: out of range', () => {
  expect(() => IDL.decode([IDL.Bool], Buffer.from('4449444c00017eff', 'hex'))).toThrow();
});
test('18:nat: 0', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017d00', 'hex'))).toEqual(expect.anything());
});
test('19:nat: 1', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017d01', 'hex'))).toEqual(expect.anything());
});
test('20:nat: 0x7f', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017d7f', 'hex'))).toEqual(expect.anything());
});
test('21:nat: leb (two bytes)', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017d8001', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('22:nat: leb (two bytes, all bits)', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017dff7f', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('23:nat: leb too short', () => {
  expect(() => IDL.decode([IDL.Nat], Buffer.from('4449444c00017d80', 'hex'))).toThrow();
});
test('24:nat: leb overlong', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017d8000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('25:nat: leb overlong', () => {
  expect(IDL.decode([IDL.Nat], Buffer.from('4449444c00017dff00', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('26:int: 0', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c00', 'hex'))).toEqual(expect.anything());
});
test('27:int: 1', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c01', 'hex'))).toEqual(expect.anything());
});
test('28:int: -1', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c7f', 'hex'))).toEqual(expect.anything());
});
test('29:int: -64', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c40', 'hex'))).toEqual(expect.anything());
});
test('30:int: leb (two bytes)', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c8001', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('31:int: leb too short', () => {
  expect(() => IDL.decode([IDL.Int], Buffer.from('4449444c00017c80', 'hex'))).toThrow();
});
test('32:int: leb overlong (0s)', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c8000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('33:int: leb overlong (1s)', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017cff7f', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('34:int: leb not overlong when signed', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017cff00', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('35:int: leb not overlong when signed', () => {
  expect(IDL.decode([IDL.Int], Buffer.from('4449444c00017c807f', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('36:nat8: 0', () => {
  expect(IDL.decode([IDL.Nat8], Buffer.from('4449444c00017b00', 'hex'))).toEqual(expect.anything());
});
test('37:nat8: 1', () => {
  expect(IDL.decode([IDL.Nat8], Buffer.from('4449444c00017b01', 'hex'))).toEqual(expect.anything());
});
test('38:nat8: 255', () => {
  expect(IDL.decode([IDL.Nat8], Buffer.from('4449444c00017bff', 'hex'))).toEqual(expect.anything());
});
test('39:nat8: too short', () => {
  expect(() => IDL.decode([IDL.Nat8], Buffer.from('4449444c00017b', 'hex'))).toThrow();
});
test('40:nat16: 0', () => {
  expect(IDL.decode([IDL.Nat16], Buffer.from('4449444c00017a0000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('41:nat16: 1', () => {
  expect(IDL.decode([IDL.Nat16], Buffer.from('4449444c00017a0100', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('42:nat16: 255', () => {
  expect(IDL.decode([IDL.Nat16], Buffer.from('4449444c00017aff00', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('43:nat16: 256', () => {
  expect(IDL.decode([IDL.Nat16], Buffer.from('4449444c00017a0001', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('44:nat16: 65535', () => {
  expect(IDL.decode([IDL.Nat16], Buffer.from('4449444c00017affff', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('45:nat16: too short', () => {
  expect(() => IDL.decode([IDL.Nat16], Buffer.from('4449444c00017a', 'hex'))).toThrow();
});
test('46:nat16: too short', () => {
  expect(() => IDL.decode([IDL.Nat16], Buffer.from('4449444c00017a00', 'hex'))).toThrow();
});
test('47:nat32: 0', () => {
  expect(IDL.decode([IDL.Nat32], Buffer.from('4449444c00017900000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('48:nat32: 1', () => {
  expect(IDL.decode([IDL.Nat32], Buffer.from('4449444c00017901000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('49:nat32: 255', () => {
  expect(IDL.decode([IDL.Nat32], Buffer.from('4449444c000179ff000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('50:nat32: 256', () => {
  expect(IDL.decode([IDL.Nat32], Buffer.from('4449444c00017900010000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('51:nat32: 65535', () => {
  expect(IDL.decode([IDL.Nat32], Buffer.from('4449444c000179ffff0000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('52:nat32: 4294967295', () => {
  expect(IDL.decode([IDL.Nat32], Buffer.from('4449444c000179ffffffff', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('53:nat32: too short', () => {
  expect(() => IDL.decode([IDL.Nat32], Buffer.from('4449444c000179', 'hex'))).toThrow();
});
test('54:nat32: too short', () => {
  expect(() => IDL.decode([IDL.Nat32], Buffer.from('4449444c00017900', 'hex'))).toThrow();
});
test('55:nat32: too short', () => {
  expect(() => IDL.decode([IDL.Nat32], Buffer.from('4449444c0001790000', 'hex'))).toThrow();
});
test('56:nat32: too short', () => {
  expect(() => IDL.decode([IDL.Nat32], Buffer.from('4449444c000179000000', 'hex'))).toThrow();
});
test('57:nat64: 0', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c0001780000000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('58:nat64: 1', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c0001780100000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('59:nat64: 255', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c000178ff00000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('60:nat64: 256', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c0001780001000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('61:nat64: 65535', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c000178ffff000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('62:nat64: 4294967295', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c000178ffffffff00000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('63:nat64: 18446744073709551615', () => {
  expect(IDL.decode([IDL.Nat64], Buffer.from('4449444c000178ffffffffffffffff', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('64:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c000178', 'hex'))).toThrow();
});
test('65:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c00017800', 'hex'))).toThrow();
});
test('66:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c0001780000', 'hex'))).toThrow();
});
test('67:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c000178000000', 'hex'))).toThrow();
});
test('68:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c00017800000000', 'hex'))).toThrow();
});
test('69:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c0001780000000000', 'hex'))).toThrow();
});
test('70:nat64: too short', () => {
  expect(() => IDL.decode([IDL.Nat64], Buffer.from('4449444c000178000000000000', 'hex'))).toThrow();
});
test('71:nat64: too short', () => {
  expect(() =>
    IDL.decode([IDL.Nat64], Buffer.from('4449444c00017800000000000000', 'hex')),
  ).toThrow();
});
test('72:int8: 0', () => {
  expect(IDL.decode([IDL.Int8], Buffer.from('4449444c00017700', 'hex'))).toEqual(expect.anything());
});
test('73:int8: 1', () => {
  expect(IDL.decode([IDL.Int8], Buffer.from('4449444c00017701', 'hex'))).toEqual(expect.anything());
});
test('74:int8: -1', () => {
  expect(IDL.decode([IDL.Int8], Buffer.from('4449444c000177ff', 'hex'))).toEqual(expect.anything());
});
test('75:int8: too short', () => {
  expect(() => IDL.decode([IDL.Int8], Buffer.from('4449444c000177', 'hex'))).toThrow();
});
test('76:int16: 0', () => {
  expect(IDL.decode([IDL.Int16], Buffer.from('4449444c0001760000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('77:int16: 1', () => {
  expect(IDL.decode([IDL.Int16], Buffer.from('4449444c0001760100', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('78:int16: 255', () => {
  expect(IDL.decode([IDL.Int16], Buffer.from('4449444c000176ff00', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('79:int16: 256', () => {
  expect(IDL.decode([IDL.Int16], Buffer.from('4449444c0001760001', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('80:int16: -1', () => {
  expect(IDL.decode([IDL.Int16], Buffer.from('4449444c000176ffff', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('81:int16: too short', () => {
  expect(() => IDL.decode([IDL.Int16], Buffer.from('4449444c000176', 'hex'))).toThrow();
});
test('82:int16: too short', () => {
  expect(() => IDL.decode([IDL.Int16], Buffer.from('4449444c00017600', 'hex'))).toThrow();
});
test('83:int32: 0', () => {
  expect(IDL.decode([IDL.Int32], Buffer.from('4449444c00017500000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('84:int32: 1', () => {
  expect(IDL.decode([IDL.Int32], Buffer.from('4449444c00017501000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('85:int32: 255', () => {
  expect(IDL.decode([IDL.Int32], Buffer.from('4449444c000175ff000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('86:int32: 256', () => {
  expect(IDL.decode([IDL.Int32], Buffer.from('4449444c00017500010000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('87:int32: 65535', () => {
  expect(IDL.decode([IDL.Int32], Buffer.from('4449444c000175ffff0000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('88:int32: -1', () => {
  expect(IDL.decode([IDL.Int32], Buffer.from('4449444c000175ffffffff', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('89:int32: too short', () => {
  expect(() => IDL.decode([IDL.Int32], Buffer.from('4449444c000175', 'hex'))).toThrow();
});
test('90:int32: too short', () => {
  expect(() => IDL.decode([IDL.Int32], Buffer.from('4449444c00017500', 'hex'))).toThrow();
});
test('91:int32: too short', () => {
  expect(() => IDL.decode([IDL.Int32], Buffer.from('4449444c0001750000', 'hex'))).toThrow();
});
test('92:int32: too short', () => {
  expect(() => IDL.decode([IDL.Int32], Buffer.from('4449444c000175000000', 'hex'))).toThrow();
});
test('93:int64: 0', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c0001740000000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('94:int64: 1', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c0001740100000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('95:int64: 255', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c000174ff00000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('96:int64: 256', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c0001740001000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('97:int64: 65535', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c000174ffff000000000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('98:int64: 4294967295', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c000174ffffffff00000000', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('99:int64: -1', () => {
  expect(IDL.decode([IDL.Int64], Buffer.from('4449444c000174ffffffffffffffff', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('100:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c000174', 'hex'))).toThrow();
});
test('101:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c00017400', 'hex'))).toThrow();
});
test('102:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c0001740000', 'hex'))).toThrow();
});
test('103:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c000174000000', 'hex'))).toThrow();
});
test('104:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c00017400000000', 'hex'))).toThrow();
});
test('105:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c0001740000000000', 'hex'))).toThrow();
});
test('106:int64: too short', () => {
  expect(() => IDL.decode([IDL.Int64], Buffer.from('4449444c000174000000000000', 'hex'))).toThrow();
});
test('107:int64: too short', () => {
  expect(() =>
    IDL.decode([IDL.Int64], Buffer.from('4449444c00017400000000000000', 'hex')),
  ).toThrow();
});
test('108:text: empty string', () => {
  expect(IDL.decode([IDL.Text], Buffer.from('4449444c00017100', 'hex'))).toEqual(expect.anything());
});
test('109:text: Motoko', () => {
  expect(IDL.decode([IDL.Text], Buffer.from('4449444c000171064d6f746f6b6f', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('110:text: too long', () => {
  expect(() =>
    IDL.decode([IDL.Text], Buffer.from('4449444c000171054d6f746f6b6f', 'hex')),
  ).toThrow();
});
test('111:text: too short', () => {
  expect(() =>
    IDL.decode([IDL.Text], Buffer.from('4449444c000171074d6f746f6b6f', 'hex')),
  ).toThrow();
});
test('112:text: overlong length leb', () => {
  expect(IDL.decode([IDL.Text], Buffer.from('4449444c00017186004d6f746f6b6f', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('113:text: Unicode', () => {
  expect(IDL.decode([IDL.Text], Buffer.from('4449444c00017103e29883', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('116:text: Invalid utf8', () => {
  expect(() => IDL.decode([IDL.Text], Buffer.from('4449444c00017103e228a1', 'hex'))).toThrow();
});
test('117:text: Unicode overshoots', () => {
  expect(() => IDL.decode([IDL.Text], Buffer.from('4449444c00017102e29883', 'hex'))).toThrow();
});
test('118:text: Escape sequences', () => {
  expect(IDL.decode([IDL.Text], Buffer.from('4449444c00017106090a0d22275c', 'hex'))).toEqual(
    expect.anything(),
  );
});
test('125:cannot decode empty type', () => {
  expect(() => IDL.decode([IDL.Empty], Buffer.from('4449444c00016f', 'hex'))).toThrow();
});
