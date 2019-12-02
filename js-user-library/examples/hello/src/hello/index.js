import hello from "../../canisters/hello/main.js";

(async () => {
  try {
    const reply = await hello.greet("World");
    console.log(reply);
  } catch (error) {
    console.error(error);
  }
})();
