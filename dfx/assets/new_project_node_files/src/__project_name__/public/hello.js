import hello from 'ic:canisters/{project_name}';

window.hello = async function(name) {
  const reply = await hello.greet(name);
  document.getElementById('output').innerText = reply;
};
