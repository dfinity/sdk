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
    expect(document.body.innerHTML).toMatchInlineSnapshot('"<div><main><img src="/logo2.svg" alt="DFINITY logo"><br><br><form action="#"><label for="name">Enter your name: &nbsp;</label><input id="name" alt="Name" type="text"><button type="submit">Click Me!</button></form><section id="greeting"></section></main></div>"');
    expect(1).toEqual(1);
  });
});
