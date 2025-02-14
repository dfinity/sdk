<script setup>
import { ref } from 'vue';
import { __backend_name_ident__ } from 'declarations/__backend_name__/index';

let greeting = ref('');

async function handleSubmit(e) {
  e.preventDefault();
  const target = e.target;
  const name = target.querySelector('#name').value;
  greeting.value = await __backend_name_ident__.greet(name);
}
</script>

<template>
  <main class="container">
    <div class="card">
      <h1 id="greeting">{{ greeting }}</h1>

      <form action="#" @submit="handleSubmit">
        <label for="name">Enter your name:</label>
        <input type="text" id="name" required />
        <button type="submit">Click Me!</button>
      </form>

      <img src="/logo.svg" alt="DFINITY logo" width="256" />
    </div>
  </main>
</template>
