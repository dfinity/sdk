use crate::lib::error::*;
use futures::future::{err, ok, result, Future};
use futures::stream::Stream;
use rand::Rng;
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

#[derive(Clone)]
pub struct Client {
    client: ReqwestClient,
    url: reqwest::Url,
}

impl Client {
    pub fn new(config: ClientConfig) -> Client {
        Client {
            client: ReqwestClient::new(),
            url: reqwest::Url::parse(config.url.as_str())
                .expect("Invalid client URL.")
                .join("api/v1/")
                .unwrap(),
        }
    }

    pub fn execute(
        &self,
        request: reqwest::r#async::Request,
    ) -> impl Future<Item = reqwest::r#async::Response, Error = reqwest::Error> {
        self.client.execute(request)
    }
}

pub struct ClientConfig {
    pub url: String,
}

/// Request payloads for the /api/v1/read endpoint.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
enum ReadRequest {
    Query {
        #[serde(flatten)]
        request: CanisterQueryCall,
    },
}

/// Response payloads for the /api/v1/read endpoint.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub enum ReadResponse<A> {
    Pending,
    Replied {
        reply: A,
    },
    Rejected {
        reject_code: ReadRejectCode,
        reject_message: String,
    },
    Unknown,
}

/// Response reject codes
#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ReadRejectCode {
    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,
}

/// Request payloads for the /api/v1/submit endpoint.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "request_type")]
enum SubmitRequest {
    InstallCode {
        canister_id: CanisterId,
        module: Blob,
        arg: Blob,
        nonce: Option<Blob>,
    },
    Call {
        canister_id: CanisterId,
        method_name: String,
        arg: Blob,
        nonce: Option<Blob>,
    },
}

/// Generates a random 32 bytes of blob.
fn random_blob() -> Blob {
    let mut rng = rand::thread_rng();
    Blob(rng.gen::<[u8; 32]>().iter().cloned().collect())
}

/// A read request. Intended to remain private in favor of exposing specialized
/// functions like `query` instead.
///
/// TODO: filter the output of this function when moving to ic_http_api.
/// For example, it should never return Unknown or Pending, per the spec.
fn read<A>(
    client: Client,
    request: ReadRequest,
) -> impl Future<Item = ReadResponse<A>, Error = DfxError>
where
    A: serde::de::DeserializeOwned,
{
    result(client.url.join("read").map_err(DfxError::Url))
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

/// Ping a client and return ok if the client is started.
pub fn ping(client: Client) -> impl Future<Item = (), Error = DfxError> {
    ok(client.url.clone()).and_then(move |url| {
        let http_request = reqwest::r#async::Request::new(reqwest::Method::GET, url);

        client
            .execute(http_request)
            .map(|_| ())
            .map_err(DfxError::Reqwest)
    })
}

/// A submit request. Intended to remain private in favor of exposing specialized
/// functions like `install_code` instead.
fn submit(
    client: Client,
    request: SubmitRequest,
) -> impl Future<Item = reqwest::r#async::Response, Error = DfxError> {
    result(client.url.join("submit").map_err(DfxError::Url)).and_then(move |url| {
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
}

/// Canister query call
///
/// Canister methods that do not change the canister state in a meaningful way
/// can be executed more efficiently. This method provides that ability, and
/// returns the canister’s response directly within the HTTP response.
pub fn query(
    client: Client,
    canister_id: CanisterId,
    method_name: String,
    arg: Option<Blob>,
) -> impl Future<Item = ReadResponse<QueryResponseReply>, Error = DfxError> {
    read(
        client,
        ReadRequest::Query {
            request: CanisterQueryCall {
                canister_id,
                method_name,
                arg: arg.unwrap_or_else(|| Blob(vec![])),
            },
        },
    )
}

/// Canister Install call
pub fn install_code(
    client: Client,
    canister_id: CanisterId,
    module: Blob,
    arg: Option<Blob>,
) -> impl Future<Item = (), Error = DfxError> {
    submit(
        client,
        SubmitRequest::InstallCode {
            canister_id,
            module,
            arg: arg.unwrap_or_else(|| Blob(vec![])),
            nonce: Some(random_blob()),
        },
    )
    .and_then(|response| {
        result(
            response
                .error_for_status()
                .map(|_| ())
                .map_err(DfxError::from),
        )
    })
}

/// Canister call
///
/// Canister methods that can change the canister state. This return right away, and cannot wait
/// for the canister to be done.
pub fn call(
    client: Client,
    canister_id: CanisterId,
    method_name: String,
    arg: Option<Blob>,
) -> impl Future<Item = (), Error = DfxError> {
    submit(
        client,
        SubmitRequest::Call {
            canister_id,
            method_name,
            arg: arg.unwrap_or_else(|| Blob(vec![])),
            nonce: Some(random_blob()),
        },
    )
    .and_then(|response| {
        result(
            response
                .error_for_status()
                .map(|_| ())
                .map_err(DfxError::from),
        )
    })
}

/// A canister query call request payload
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanisterQueryCall {
    pub canister_id: CanisterId,
    pub method_name: String,
    pub arg: Blob,
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
        let arg = Blob(vec![]);

        let request = ReadRequest::Query {
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
                (Value::Text("arg".to_string()), Value::Bytes(vec![])),
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

        let actual: ReadResponse<QueryResponseReply> =
            serde_cbor::from_slice(&serde_cbor::to_vec(&response).unwrap()).unwrap();

        let expected = ReadResponse::Replied {
            reply: QueryResponseReply {
                arg: Blob(arg.clone()),
            },
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn query_response_replied() {
        let _ = env_logger::try_init();

        let response = ReadResponse::Replied {
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
            url: mockito::server_url(),
        });

        let query = query(client, 1, "main".to_string(), Some(Blob(vec![])));

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

        let actual: ReadResponse<QueryResponseReply> =
            serde_cbor::from_slice(&serde_cbor::to_vec(&response).unwrap()).unwrap();

        let expected: ReadResponse<QueryResponseReply> = ReadResponse::Rejected {
            reject_code: ReadRejectCode::SysFatal,
            reject_message: reject_message.clone(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn query_response_rejected() {
        let _ = env_logger::try_init();

        let response = ReadResponse::Rejected {
            reject_code: ReadRejectCode::SysFatal,
            reject_message: "Fatal error".to_string(),
        };

        let _m = mock("POST", "/api/v1/read")
            .with_status(200)
            .with_header("content-type", "application/cbor")
            .with_body(serde_cbor::to_vec(&response).unwrap())
            .create();

        let client = Client::new(ClientConfig {
            url: mockito::server_url(),
        });

        let query = query(client, 1, "main".to_string(), Some(Blob(vec![])));

        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let result = runtime.block_on(query);

        _m.assert();

        match result {
            Ok(r) => assert_eq!(r, response),
            Err(e) => assert!(false, format!("{:#?}", e)),
        }
    }

    #[test]
    fn install_code_request_serialization() {
        use serde_cbor::Value;
        use std::convert::TryInto;

        let canister_id = 1;
        let module = Blob(vec![1]);
        let arg = Blob(vec![2]);

        let request = SubmitRequest::InstallCode {
            canister_id,
            module,
            arg,
            nonce: None,
        };

        let actual: Value = serde_cbor::from_slice(&serde_cbor::to_vec(&request).unwrap()).unwrap();

        let expected = Value::Map(
            vec![
                (
                    Value::Text("request_type".to_string()),
                    Value::Text("install_code".to_string()),
                ),
                (
                    Value::Text("canister_id".to_string()),
                    Value::Integer(canister_id.try_into().unwrap()),
                ),
                (Value::Text("module".to_string()), Value::Bytes(vec![1])),
                (Value::Text("arg".to_string()), Value::Bytes(vec![2])),
                (Value::Text("nonce".to_string()), Value::Null),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn install_code_response_replied() {
        let _ = env_logger::try_init();

        let _m = mock("POST", "/api/v1/submit")
            .with_status(200)
            .with_header("content-type", "application/cbor")
            .create();

        let client = Client::new(ClientConfig {
            url: mockito::server_url(),
        });

        let future = install_code(client, 1, Blob(vec![1]), None);

        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let result = runtime.block_on(future);

        _m.assert();

        match result {
            Ok(()) => {}
            Err(e) => assert!(false, format!("{:#?}", e)),
        }
    }

    #[test]
    fn install_code_response_rejected() {
        let _ = env_logger::try_init();

        let _m = mock("POST", "/api/v1/submit")
            .with_status(400)
            .with_header("content-type", "application/cbor")
            .create();

        let client = Client::new(ClientConfig {
            url: mockito::server_url(),
        });

        let future = install_code(client, 1, Blob(vec![1]), None);

        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let result = runtime.block_on(future);

        _m.assert();

        match result {
            Ok(()) => assert!(false, "Install succeeded."),
            Err(e) => match e {
                DfxError::Reqwest(_err) => (),
                _ => assert!(false, format!("{:#?}", e)),
            },
        }
    }
}
