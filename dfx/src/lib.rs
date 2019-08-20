use futures::future::{Future, ok, err, result};
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

pub trait Client {
    fn execute(client: impl Client, request: reqwest::Request) -> impl Future<Item=reqwest::r#async::Response, Error=DfxError>;
}

fn read(client: impl Client, message: Message) -> impl Future<Item=reqwest::r#async::Response, Error=DfxError> {
    return result(reqwest::Url::parse("http://localhost/api/v1/read"))
        .and_then(|url| {
            let request = reqwest::Request::new(reqwest::Method::POST, url);
            let mut headers = request.headers_mut();
            headers.insert(reqwest::header::CONTENT_TYPE, "application/cbor".parse().unwrap());
            // .body(serde_cbor::to_vec(&message).unwrap())
            // .build();

            // .map_err(DfxError::Reqwest);
            return result(Client::execute(client, request).map_err(DfxError::Reqwest));
        });
}

pub fn query(client: impl Client, message: CanisterQueryCall) -> impl Future<Item=Response<String>, Error=DfxError> {
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
