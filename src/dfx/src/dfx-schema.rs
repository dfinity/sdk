use crate::config::dfinity::ConfigInterface;
use schemars::schema_for;

mod actors;
mod commands;
mod config;
mod lib;
mod util;

fn main() {
    let schema = schema_for!(ConfigInterface);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
