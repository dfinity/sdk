import { Actor, HttpAgent } from '@dfinity/agent';
import { IDL as {project_name}_idl, canisterId as {project_name}_id } from '@ic/canisters/{project_name}';

const agent = new HttpAgent();
const {project_name} = Actor.createActor({project_name}_idl, { agent, canisterId: {project_name}_id })

document.getElementById("clickMeBtn").addEventListener("click", async () => {
  const name = document.getElementById("name").value.toString();
  const greeting = await {project_name}.greet(name);

  document.getElementById("greeting").innerText = greeting;
});
