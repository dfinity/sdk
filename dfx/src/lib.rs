use futures::future::{err, ok, result, Future};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockito;

type Blob = String;
type CanisterId = u64;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanisterQueryCall {
    pub canister_id: CanisterId,
    pub method_name: String,
    pub arg: Option<Blob>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "message_type")]
enum Message {
    Query {
        #[serde(flatten)]
        message: CanisterQueryCall,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Accepted,
    Replied,
    Rejected,
    Unknown,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectCode {
    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response<A> {
    pub status: Status,
    pub reply: Option<A>,
    pub reject_code: Option<RejectCode>,
    pub reject_message: Option<String>,
}

#[derive(Debug)]
pub enum DfxError {
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
    Url(reqwest::UrlError),
}

impl From<reqwest::Error> for DfxError {
    fn from(err: reqwest::Error) -> DfxError {
        DfxError::Reqwest(err)
    }
}

impl From<reqwest::UrlError> for DfxError {
    fn from(err: reqwest::UrlError) -> DfxError {
        DfxError::Url(err)
    }
}

// TODO: move to own file, use conditional compilation for testing
pub struct Client {
    client: reqwest::r#async::Client,
    url: String,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::r#async::Client::new(),

            #[cfg(not(test))]
            // url: "http://10.129.10.139:8080".to_string(),
            url: "http://localhost:8080".to_string(),

            #[cfg(test)]
            url: mockito::server_url(),
        }
    }

    pub fn execute(
        &self,
        request: reqwest::r#async::Request,
    ) -> impl Future<Item = reqwest::r#async::Response, Error = reqwest::Error> {
        self.client.execute(request)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

fn read<A>(client: Client, message: Message) -> impl Future<Item = Response<A>, Error = DfxError>
where
    A: serde::de::DeserializeOwned,
{
    let endpoint = format!("{}/api/v1/read", client.url);
    let parsed = reqwest::Url::parse(&endpoint).map_err(DfxError::Url);
    result(parsed)
        .and_then(move |url| {
            let mut request = reqwest::r#async::Request::new(reqwest::Method::POST, url);
            let headers = request.headers_mut();
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/cbor".parse().unwrap(),
            );
            let body = request.body_mut();
            body.get_or_insert(reqwest::r#async::Body::from(
                serde_cbor::to_vec(&message).unwrap(),
            ));
            client.execute(request).map_err(DfxError::Reqwest)
        })
        .and_then(|mut res| res.text().map_err(DfxError::Reqwest))
        .and_then(|text| {
            let bytes = text.as_bytes();
            match serde_cbor::from_slice(bytes) {
                Ok(r) => ok(r),
                Err(e) => err(DfxError::SerdeCbor(e)),
            }
        })
}

pub fn query(
    client: Client,
    message: CanisterQueryCall,
) -> impl Future<Item = Response<String>, Error = DfxError> {
    read::<String>(client, Message::Query { message })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[test]
    fn query_hello_world() {
        let _ = env_logger::try_init();

        let response = Response {
            status: Status::Replied,
            reply: Some("Hello World".to_string()),
            reject_code: None,
            reject_message: None,
        };

        let _m = mock("POST", "/api/v1/read")
            .with_status(200)
            .with_header("content-type", "application/cbor")
            .with_body(serde_cbor::to_vec(&response).unwrap())
            .create();

        let client = Client::new();

        let query = query(
            client,
            CanisterQueryCall {
                canister_id: 0,
                method_name: "main".to_string(),
                arg: None,
            },
        );

        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let result = runtime.block_on(query);

        _m.assert();

        match result {
            Ok(r) => assert_eq!(r, response),
            Err(e) => assert!(false, format!("{:#?}", e)),
        }
    }
}
