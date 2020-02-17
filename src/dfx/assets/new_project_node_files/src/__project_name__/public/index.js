import {project_name} from 'ic:canisters/{project_name}';

{project_name}.greet(window.prompt("Enter your name:")).then(greeting => {
  window.alert(greeting);
});
