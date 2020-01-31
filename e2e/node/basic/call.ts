import counterFactory from '../utils/canisters/counter';

test('can call a canister', async () => {
  let counter = await counterFactory();

  expect(+(await counter.read())).toEqual(0);
  expect(+(await counter.inc_read())).toEqual(1);
  await counter.write(10);
  expect(+(await counter.read())).toEqual(10);
  expect(+(await counter.inc_read())).toEqual(11);
});
