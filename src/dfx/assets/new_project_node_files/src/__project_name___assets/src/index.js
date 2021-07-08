import { {project_name} } from "../../declarations/{project_name}";

document.getElementById("clickMeBtn").addEventListener("click", async () => {
  const name = document.getElementById("name").value.toString();
  // Interact with {project_name} actor, calling the greet method
  const greeting = await {project_name}.greet(name);

  document.getElementById("greeting").innerText = greeting;
});
