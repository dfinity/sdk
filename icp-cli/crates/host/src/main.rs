mod bindings;
mod host;
mod prettify;

use prettify::*;

pub fn main() {
    let mut prettify = Prettify::new("target/wasm32-wasip2/release/plugin.wasm").unwrap();
    let r = prettify.prettify("We will prettify this with a plugin").unwrap();
    println!("{}", r);
}
