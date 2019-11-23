import hello from '../../../canisters/hello/main.js';

window.hello = async function(name) {
  const reply = await hello.main(name);
  document.getElementById('output').innerText = reply;
};
