import { {project_name} } from "../../declarations/{project_name}";

document.getElementById("clickMeBtn").addEventListener("click", async () => {
  const name = document.getElementById("name").value.toString();
  const greeting = await {project_name}.greet(name);

  document.getElementById("greeting").innerText = greeting;
});
