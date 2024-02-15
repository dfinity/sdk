import { describe, expect, it } from 'vitest';
import App from '../App';

describe('App', () => {
  it('renders as expected', () => {
    const root = document.createElement('div');
    root.id = 'root';
    document.body.appendChild(root);
    new App();

    expect(root.querySelector('main')).toBeTruthy();
  });
});
