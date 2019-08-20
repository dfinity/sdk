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
    Url(reqwest::UrlError),
}

impl From<reqwest::Error> for DfxError {
    fn from(err: reqwest::Error) -> DfxError {
        return DfxError::Reqwest(err);
    }
}

impl From<reqwest::UrlError> for DfxError {
    fn from(err: reqwest::UrlError) -> DfxError {
        return DfxError::Url(err);
    }
}

pub trait Client {
    fn execute(&self, request: reqwest::r#async::Request) -> Box<dyn Future<Item=reqwest::r#async::Response, Error=reqwest::Error> + Send>;
}

pub struct AsyncClient {
    client: reqwest::r#async::Client,
}

impl AsyncClient {
    pub fn new() -> AsyncClient {
        AsyncClient {
            client: reqwest::r#async::Client::new(),
        }
    }
}

impl Client for AsyncClient {
    fn execute(&self, request: reqwest::r#async::Request) -> Box<dyn Future<Item=reqwest::r#async::Response, Error=reqwest::Error> + Send> {
        return Box::new(self.client.execute(request));
    }
}

fn read(client: &impl Client, message: Message) -> impl Future<Item=reqwest::r#async::Response, Error=DfxError> + '_ {
    let parsed = reqwest::Url::parse("http://localhost/api/v1/read").map_err(DfxError::Url);
    return result(parsed)
        .and_then(move |url| {
            let mut request = reqwest::r#async::Request::new(reqwest::Method::POST, url);
            let headers = request.headers_mut();
            headers.insert(reqwest::header::CONTENT_TYPE, "application/cbor".parse().unwrap());
            // .body(serde_cbor::to_vec(&message).unwrap())
            return client.execute(request).map_err(DfxError::Reqwest);
        });
}

pub fn query(client: &impl Client, message: CanisterQueryCall) -> impl Future<Item=Response<String>, Error=DfxError> + '_ {
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
