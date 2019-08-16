extern crate serde;

use serde::{Deserialize, Serialize};

type Blob = String;
type CanisterId = u64;

#[derive(Debug, Serialize, Deserialize)]
struct CanisterQueryCall {
    canister_id: CanisterId,
    method_name: String,
    arg: Option<Blob>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "message_type")]
enum Message {
    Query {
        #[serde(flatten)]
        message: CanisterQueryCall,
    },
}

// TODO: https://github.com/dfinity-lab/dfinity/blob/ebeefdc6cf4a1d2c710fce91e0451dbfe0d75d1d/docs/spec/public/index.adoc#canister-query-call
// * Use `reqwest` to make a canister query call to /api/v1/read
// * Define a type containing response fields
// * if response.status == replied then println!(status.replied) else exit(1)

fn main() {
    println!("dfx");
    ::std::process::exit(1);
}
