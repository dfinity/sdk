use super::bindings::{icp::host::host, export, exports::icp::host::prettify_plugin};

// This is our implementation of the "prettify" plugin type
// (=WIT world)

// We shall make the content pretty by quoting all words

// Gorgeous!

pub struct QuotePrettifyPlugin;

export!(QuotePrettifyPlugin);

impl prettify_plugin::Guest for QuotePrettifyPlugin {
    fn prettify(content: String) -> String {
        host::log("thank you for using the quote prettify plugin!");
        let words = content.split(" ");
        let words: Vec<String> = words.map(|word| format!("{:?}", word)).collect();
        words.join(" ")
    }
}
