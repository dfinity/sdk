const js = import("./node_modules/idl-wasm/idl_wasm.js");
js.then(js => {
  try {
    const bytes = js.js_encode([42,false]);
    console.log(bytes);
    //const bytes = js.encode('(42, record { ok=opt 42; label="test" })');
    //console.log(bytes);
    const json = js.decode(bytes);
    console.log(json);
    /*
    const values = JSON.parse(json);
    console.log(values.args[0].Int);
    console.log(values.args[1].Record[1].val.Text);
    */
  } catch (e) {
    console.error(e);
  }
});
