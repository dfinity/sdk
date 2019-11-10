const js = import("./node_modules/idl-wasm/idl_wasm.js");
js.then(js => {
  try {
    const bytes = js.js_encode([42,false, "哈哈", { 'ok': 42, 123: 'test' }, [[1,2,3],[4,5]]]);
    console.log(bytes);
    const args = js.js_decode(bytes);
    console.log(args);
    //const bytes = js.encode('(42, record { ok=opt 42; label="test" })');
    //console.log(bytes);
  } catch (e) {
    console.error(e);
  }
});
