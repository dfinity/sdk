import { hello } from '../src/hello/main';

window.hello = async function(name) {
  const response = await hello.main(name);
  console.log(response);
};
