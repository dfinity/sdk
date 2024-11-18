<script>
  import "../index.scss";
  import { backend } from "$lib/canisters";

  let greeting = "";

  async function onSubmit(event) {
    const name = event.target.name.value;
    greeting = await backend.greet(name);
    return false;
  }
</script>

<main class="container">
  <div class="card">
    <h1>{greeting}</h1>

    <form action="#" on:submit|preventDefault={onSubmit}>
      <label for="name">Enter your name:</label>
      <input type="text" id="name" required />
      <button type="submit">Click Me!</button>
    </form>

    <img src="/logo2.svg" alt="DFINITY logo" width="256" />
  </div>
</main>
