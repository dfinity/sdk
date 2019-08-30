use crate::lib::error::*;
use futures::future::{err, ok, result, Future};
use futures::stream::Stream;
use reqwest::r#async::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// A binary "blob", i.e. a byte array
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
// XXX: We newtype and make sure that serde uses `serde_bytes`, otherwise the `Vec<u8>` is
// deserialized as a sequence (array) of bytes, whereas we want an actual CBOR "byte array", e.g. a
// bytestring
pub struct Blob(#[serde(with = "serde_bytes")] pub Vec<u8>);

type CanisterId = u64;

pub struct Client {
    client: ReqwestClient,
    host: String,
}

impl Client {
    pub fn new(config: ClientConfig) -> Client {
        Client {
            client: ReqwestClient::new(),
            host: config.host,
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
    fn default() -> Client {
        Client::new(Default::default())
    }
}

pub struct ClientConfig {
    pub host: String,
}

impl Default for ClientConfig {
    fn default() -> ClientConfig {
        ClientConfig {
            host: "http://localhost:8080".to_string(),
        }
    }
}

/// Request payloads
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
enum Request {
    Query {
        #[serde(flatten)]
        request: CanisterQueryCall,
    },
}

/// Response payloads
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub enum Response<A> {
    Accepted,
    Replied {
        reply: A,
    },
    Rejected {
        reject_code: RejectCode,
        reject_message: String,
    },
    Unknown,
}

/// Response reject codes
#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RejectCode {
    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,
}

/// A read request. Intended to remain private in favor of exposing specialized
/// functions like `query` instead.
fn read<'a, A: 'a>(client: &'a Client, request: Request) -> impl Future<Item = Response<A>, Error = DfxError> + 'a
where
    A: serde::de::DeserializeOwned,
{
    let endpoint = format!("{}/api/v1/read", client.host);
    let parsed = reqwest::Url::parse(&endpoint).map_err(DfxError::Url);
    result(parsed)
        .and_then(move |url| {
            let mut http_request = reqwest::r#async::Request::new(reqwest::Method::POST, url);
            let headers = http_request.headers_mut();
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/cbor".parse().unwrap(),
            );
            let body = http_request.body_mut();
            body.get_or_insert(reqwest::r#async::Body::from(
                serde_cbor::to_vec(&request).unwrap(),
            ));
            client.execute(http_request).map_err(DfxError::Reqwest)
        })
        .and_then(|res| res.into_body().concat2().map_err(DfxError::Reqwest))
        .and_then(|buf| match serde_cbor::from_slice(&buf) {
            Ok(r) => ok(r),
            Err(e) => err(DfxError::SerdeCbor(e)),
        })
}

/// Canister query call
///
/// Canister methods that do not change the canister state in a meaningful way
/// can be executed more efficiently. This method provides that ability, and
/// returns the canister’s response directly within the HTTP response.
pub fn query<'a>(
    client: &'a Client,
    request: CanisterQueryCall,
) -> impl Future<Item = Response<QueryResponseReply>, Error = DfxError> + 'a {
    read(client, Request::Query { request })
}

/// A canister query call request payload
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanisterQueryCall {
    pub canister_id: CanisterId,
    pub method_name: String,
    pub arg: Option<Blob>,
}

/// A canister query call response payload
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResponseReply {
    pub arg: Blob,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use mockito::mock;

    #[test]
    fn query_request_serialization() {
        use serde_cbor::Value;
        use std::convert::TryInto;

        let canister_id = 1;
        let method_name = "main".to_string();
        let arg = None;

        let request = Request::Query {
            request: CanisterQueryCall {
                canister_id,
                method_name: method_name.clone(),
                arg,
            },
        };

        let actual: Value = serde_cbor::from_slice(&serde_cbor::to_vec(&request).unwrap()).unwrap();

        let expected = Value::Map(
            vec![
                (
                    Value::Text("request_type".to_string()),
                    Value::Text("query".to_string()),
                ),
                (
                    Value::Text("canister_id".to_string()),
                    Value::Integer(canister_id.try_into().unwrap()),
                ),
                (
                    Value::Text("method_name".to_string()),
                    Value::Text(method_name.clone()),
                ),
                (Value::Text("arg".to_string()), Value::Null),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn query_response_replied_deserialization() {
        use serde_cbor::Value;

        let arg = Vec::from("Hello World");

        let response = Value::Map(
            vec![
                (
                    Value::Text("status".to_string()),
                    Value::Text("replied".to_string()),
                ),
                (
                    Value::Text("reply".to_string()),
                    Value::Map(
                        vec![(Value::Text("arg".to_string()), Value::Bytes(arg.clone()))]
                            .into_iter()
                            .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        );

        let actual: Response<QueryResponseReply> =
            serde_cbor::from_slice(&serde_cbor::to_vec(&response).unwrap()).unwrap();

        let expected = Response::Replied {
            reply: QueryResponseReply {
                arg: Blob(arg.clone()),
            },
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn query_response_replied() {
        let _ = env_logger::try_init();

        let response = Response::Replied {
            reply: QueryResponseReply {
                arg: Blob(Vec::from("Hello World")),
            },
        };

        let _m = mock("POST", "/api/v1/read")
            .with_status(200)
            .with_header("content-type", "application/cbor")
            .with_body(serde_cbor::to_vec(&response).unwrap())
            .create();

        let client = Client::new(ClientConfig {
            host: mockito::server_url(),
        });

        let query = query(
            Box::leak(Box::new(client)),
            CanisterQueryCall {
                canister_id: 1,
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

    #[test]
    fn query_response_rejected_deserialization() {
        use serde_cbor::Value;

        let reject_message = "Fatal error".to_string();

        let response = Value::Map(
            vec![
                (
                    Value::Text("status".to_string()),
                    Value::Text("rejected".to_string()),
                ),
                (Value::Text("reject_code".to_string()), Value::Integer(1)),
                (
                    Value::Text("reject_message".to_string()),
                    Value::Text(reject_message.clone()),
                ),
            ]
            .into_iter()
            .collect(),
        );

        let actual: Response<QueryResponseReply> =
            serde_cbor::from_slice(&serde_cbor::to_vec(&response).unwrap()).unwrap();

        let expected: Response<QueryResponseReply> = Response::Rejected {
            reject_code: RejectCode::SysFatal,
            reject_message: reject_message.clone(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn query_response_rejected() {
        let _ = env_logger::try_init();

        let response = Response::Rejected {
            reject_code: RejectCode::SysFatal,
            reject_message: "Fatal error".to_string(),
        };

        let _m = mock("POST", "/api/v1/read")
            .with_status(200)
            .with_header("content-type", "application/cbor")
            .with_body(serde_cbor::to_vec(&response).unwrap())
            .create();

        let client = Client::new(ClientConfig {
            host: mockito::server_url(),
        });

        let query = query(
            Box::leak(Box::new(client)),
            CanisterQueryCall {
                canister_id: 1,
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
