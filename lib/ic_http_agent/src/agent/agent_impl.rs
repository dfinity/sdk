use crate::agent::agent_error::AgentError;
use crate::{Blob, CanisterId};
use reqwest::header::HeaderMap;
use reqwest::Client;
use reqwest::Method;
use serde::{de, Deserialize, Serialize};
use serde_idl::{Encode, IDLType};

pub struct Agent {
    url: reqwest::Url,
    client: reqwest::Client,
}

/// Request payloads for the /api/v1/read endpoint.
/// This never needs to be deserialized.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "request_type")]
pub(crate) enum ReadRequest<'a> {
    Query {
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
    },
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub(crate) enum ReadResponse<A> {
    Replied {
        reply: Option<A>,
    },
    Rejected {
        reject_code: u16,
        reject_message: String,
    },
}

impl Agent {
    pub fn with_url<T: AsRef<str>>(url: T) -> Result<Agent, AgentError> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/cbor".parse().unwrap(),
        );

        Ok(Agent {
            url: reqwest::Url::parse(url.as_ref())
                .and_then(|url| url.join("api/v1/"))
                .map_err(|_| AgentError::InvalidClientUrl(String::from(url.as_ref())))?,
            client: Client::builder().default_headers(default_headers).build()?,
        })
    }

    async fn _read(&self, request: ReadRequest<'_>) -> Result<ReadResponse<Blob>, AgentError> {
        let record = serde_cbor::to_vec(&request)?;
        let url = self.url.join("read")?;

        let mut http_request = reqwest::Request::new(Method::POST, url);
        http_request
            .body_mut()
            .get_or_insert(reqwest::Body::from(record));

        let bytes = self.client.execute(http_request).await?.bytes().await?;
        println!("{}", hex::encode(&bytes));
        println!("{:?}", serde_cbor::from_slice::<ReadResponse<Blob>>(&bytes));
        serde_cbor::from_slice::<ReadResponse<Blob>>(&bytes).map_err(AgentError::InvalidData)
    }

    /// The simplest for of query; sends a Blob and will return a Blob. The encoding is
    /// left as an exercise to the user.
    pub async fn query_blob<'a>(
        &self,
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a Blob,
    ) -> Result<Blob, AgentError> {
        self._read(ReadRequest::Query {
            canister_id,
            method_name,
            arg,
        })
        .await
        .and_then(|response| match response {
            ReadResponse::Replied { reply } => Ok(reply.unwrap_or_else(Blob::empty)),
            ReadResponse::Rejected {
                reject_code,
                reject_message,
            } => Err(AgentError::ClientError(reject_code, reject_message)),
        })
    }

    pub async fn query<'a, TArg: IDLType, TResult: de::DeserializeOwned>(
        &self,
        canister_id: &'a CanisterId,
        method_name: &'a str,
        arg: &'a TArg,
    ) -> Result<TResult, AgentError> {
        self.query_blob(canister_id, method_name, &Blob::from(Encode!(arg)))
            .await
            .map(|result| {
                let mut de = serde_idl::de::IDLDeserialize::new(result.as_slice());
                let decoded: TResult = de.get_value().unwrap();
                de.done().map_err(AgentError::IDLDeserializationError)?;
                Ok(decoded)
            })?
    }
}
