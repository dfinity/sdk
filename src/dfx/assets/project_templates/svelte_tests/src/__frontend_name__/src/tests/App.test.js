import { test, expect } from 'vitest';
import { render, screen } from "@testing-library/svelte";
import App from '../routes/+page.svelte';

test('mount component', async () => {
  render(App, { props: {} });

  expect(screen.getByRole('main')).toMatchInlineSnapshot(`
    <main>
      <img
        alt="DFINITY logo"
        src="/logo2.svg"
      />
       
      <br />
       
      <br />
       
      <form
        action="#"
      >
        <label
          for="name"
        >
          Enter your name:  
        </label>
         
        <input
          alt="Name"
          id="name"
          type="text"
        />
         
        <button
          type="submit"
        >
          Click Me!
        </button>
      </form>
       
      <section
        id="greeting"
      >
        
      </section>
    </main>
  `);
});
