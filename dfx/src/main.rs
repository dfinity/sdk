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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Replied,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
enum RejectCode {
    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response<A> {
    status: Status,
    reply: A,
    reject_code: RejectCode,
    reject_message: String,
}

fn read(client: reqwest::Client, message: Message) -> reqwest::Result<reqwest::Response> {
    return client.post("http://localhost/api/v1/read")
        .header(reqwest::header::CONTENT_TYPE, "application/cbor")
        .body(serde_cbor::to_vec(&message).unwrap())
        .send();
}

#[derive(Debug)]
enum DfxError {
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
}

impl From<reqwest::Error> for DfxError {
    fn from(err: reqwest::Error) -> DfxError {
        return DfxError::Reqwest(err);
    }
}

type DfxResult<A> = Result<A, DfxError>;

fn query(client: reqwest::Client, message: CanisterQueryCall) -> DfxResult<Response<String>> {
    let mut res = read(client, Message::Query { message })?;
    let mut buf: Vec<u8> = vec![];
    res.copy_to(&mut buf)?;
    return serde_cbor::de::from_slice(buf.as_slice()).map_err(DfxError::SerdeCbor);
}

fn main() {
    let client = reqwest::Client::new();
    let res = query(client, CanisterQueryCall {
        canister_id: 0,
        method_name: "main".to_string(),
        arg: None,
    });
    match res {
        Ok(r) => println!("{}", r.reply),
        Err(e) => {
            println!("{:#?}", e);
            ::std::process::exit(1);
        },
    }
}
