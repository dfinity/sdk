import { hash } from './hash';

test('IDL hash', () => {
  function testHash(str: string, expected: number) {
    expect(hash(str)).toBe(expected);
  }

  testHash('', 0);
  testHash('id', 23515);
  testHash('description', 1595738364);
  testHash('short_name', 3261810734);
  testHash('Hi ☃', 1419229646);
});

