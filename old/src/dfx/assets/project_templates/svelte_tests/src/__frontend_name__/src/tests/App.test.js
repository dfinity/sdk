import { test, expect, afterEach } from 'vitest';
import App from '../routes/+page.svelte';

let host;

afterEach(() => {
  host.remove();
});

test('mount component', async () => {
  host = document.createElement('div');
  host.setAttribute('id', 'host');
  document.body.appendChild(host);
  const instance = new App({ target: host, props: {} });
  expect(instance).toBeTruthy();
  expect(host.innerHTML).toMatchInlineSnapshot(
    '"<main><img src=\\"/logo2.svg\\" alt=\\"DFINITY logo\\"> <br> <br> <form action=\\"#\\"><label for=\\"name\\">Enter your name: &nbsp;</label> <input id=\\"name\\" alt=\\"Name\\" type=\\"text\\"> <button type=\\"submit\\">Click Me!</button></form> <section id=\\"greeting\\"></section></main><!--<+page>-->"',
  );
});
