import { useState } from 'react';
import { __backend_name_ident__ } from 'declarations/__backend_name__';

function App() {
  const [greeting, setGreeting] = useState('');

  async function handleSubmit(event) {
    event.preventDefault();
    const name = event.target.elements.name.value;
    const greeting = await __backend_name_ident__.greet(name);
    setGreeting(greeting);
    return false;
  }

  return (
    <main className="container">
      <div className="card">
        <h1>{greeting}</h1>
        <form action="#" onSubmit={handleSubmit}>
          <label htmlFor="name">Enter your name:</label>
          <input type="text" id="name" required />
          <button type="submit">Click Me!</button>
        </form>
        <img src="/logo.svg" alt="DFINITY logo" width="256" />
      </div>
    </main>
  );
}

export default App;
