import {render, screen} from '@testing-library/svelte'
import { test, expect } from 'vitest';
import App from '../routes/+page.svelte';

let host;

test('mount component', async () => {
  render(App,{ target: host, props: {} });

  const main = screen.getByRole('main');
  expect(main.outerHTML).toMatchInlineSnapshot(
    '"<main><img src="/logo2.svg" alt="DFINITY logo"> <br> <br> <form action="#"><label for="name">Enter your name: &nbsp;</label> <input id="name" alt="Name" type="text"> <button type="submit">Click Me!</button></form> <section id="greeting"></section></main>"'
  );
});
