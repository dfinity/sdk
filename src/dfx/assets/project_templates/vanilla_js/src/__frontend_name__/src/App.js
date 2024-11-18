import { html, render } from 'lit-html';
import { __backend_name_ident__ } from 'declarations/__backend_name__';
import logo from './logo.svg';

class App {
  greeting = "";

  constructor() {
    this.#render();
  }

  #handleSubmit = async (e) => {
    e.preventDefault();
    const name = document.getElementById("name").value;
    this.greeting = await __backend_name_ident__.greet(name);
    this.#render();
  };

  #render() {
    let body = html`
        <main class="container">
            <div class="card">
                <h1>${this.greeting}</h1>

                <form action="#">
                    <label for="name">Enter your name:</label>
                    <input type="text" id="name" required />
                    <button type="submit">Click Me!</button>
                </form>

                <img src="${logo}" alt="DFINITY logo" width="256" />
            </div>
        </main>
    `;
    render(body, document.getElementById("root"));
    document
      .querySelector("form")
      .addEventListener("submit", this.#handleSubmit);
  }
}

export default App;
