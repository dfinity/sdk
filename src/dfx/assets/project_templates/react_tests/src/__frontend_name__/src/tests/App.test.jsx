import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from '../App';
import { StrictMode } from 'react';

describe('App', () => {
  it('renders as expected', () => {
    render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
    expect(document.body.innerHTML).toMatchInlineSnapshot('"<div><main class="container"><div class="card"><h1 id="greeting"></h1><form action="#"><label for="name">Enter your name:</label><input type="text" id="name" required=""><button type="submit">Click Me!</button></form><img src="/logo.svg" alt="DFINITY logo" width="256"></div></main></div>"');
    expect(1).toEqual(1);
  });
});
