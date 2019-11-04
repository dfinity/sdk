const js = import("./node_modules/idl-wasm/idl_wasm.js");
js.then(js => {
  alert(js.encode("(42)"));
});
