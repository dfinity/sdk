extern crate reqwest;
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

fn main() {
    let client = reqwest::Client::new();
    let message = CanisterQueryCall {
        canister_id: 0,
        method_name: "main".to_string(),
        arg: None,
    };
    let query = Message::Query { message };
    let res = client.post("/api/v1/read")
        .header(reqwest::header::CONTENT_TYPE, "application/cbor")
        .body(serde_cbor::to_vec(&query).unwrap())
        .send();
    match res {
        // TODO
        // * Check response body "status" field is "replied"
        // * Print value of response body "reply" field
        Ok(r) => println!("{}", r.status()),
        Err(_) => ::std::process::exit(1),
    }
}
