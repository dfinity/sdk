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
    '"<main class=\"container\"><div class=\"card\"><h1>{greeting}</h1><form action=\"#\"><label for=\"name\">Enter your name:</label><input type=\"text\" id=\"name\" required /><button type=\"submit\">Click Me!</button></form><img src=\"/logo.svg\" alt=\"DFINITY logo\" width=\"256\" /></div></main>"'
  );
});
