use futures::future::{Future, ok, err};
use serde::{Deserialize, Serialize};

type Blob = String;
type CanisterId = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CanisterQueryCall {
    pub canister_id: CanisterId,
    pub method_name: String,
    pub arg: Option<Blob>,
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
pub enum Status {
    Replied,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RejectCode {
    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<A> {
    pub status: Status,
    pub reply: A,
    pub reject_code: RejectCode,
    pub reject_message: String,
}

fn read(client: reqwest::r#async::Client, message: Message) -> impl Future<Item=reqwest::r#async::Response, Error=DfxError> {
    return client.post("http://localhost/api/v1/read")
        .header(reqwest::header::CONTENT_TYPE, "application/cbor")
        .body(serde_cbor::to_vec(&message).unwrap())
        .send()
        .map_err(DfxError::Reqwest);
}

#[derive(Debug)]
pub enum DfxError {
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
}

impl From<reqwest::Error> for DfxError {
    fn from(err: reqwest::Error) -> DfxError {
        return DfxError::Reqwest(err);
    }
}

pub fn query(client: reqwest::r#async::Client, message: CanisterQueryCall) -> impl Future<Item=Response<String>, Error=DfxError> {
    return read(client, Message::Query { message })
        .and_then(|mut res| {
            return res.text().map_err(DfxError::Reqwest);
        })
        .and_then(|text| {
            match serde_cbor::de::from_slice(text[..].as_bytes()) {
                Ok(r) => ok(r),
                Err(e) => err(DfxError::SerdeCbor(e)),
            }
        });
}
