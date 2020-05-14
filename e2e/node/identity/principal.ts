import { identityFactory } from '../utils/canisters';

test('has the same identity when calling a query or a call function', async () => {
  const identity = await identityFactory();

  const call = await identity.hashFromCall();
  const query = await identity.hashFromQuery();

  expect(+call).toEqual(+query);
});
