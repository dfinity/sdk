use crate::rc_bytes::RcBytes;
use crate::state_machine::{Asset, AssetEncoding};
use candid::{CandidType, Deserialize, Func, Nat};
use ic_certified_map::Hash;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

const HTTP_REDIRECT_PERMANENT: u16 = 308;

pub type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: RcBytes,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackToken {
    pub key: String,
    pub content_encoding: String,
    pub index: Nat,
    // We don't care about the sha, we just want to be backward compatible.
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum StreamingStrategy {
    Callback {
        callback: Func,
        token: StreamingCallbackToken,
    },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    pub body: RcBytes,
    pub token: Option<StreamingCallbackToken>,
}

impl StreamingCallbackToken {
    pub fn create_token(
        enc_name: &str,
        content_chunks_count: usize,
        content_sha256: [u8; 32],
        key: &str,
        chunk_index: usize,
    ) -> Option<Self> {
        if chunk_index + 1 >= content_chunks_count {
            None
        } else {
            Some(StreamingCallbackToken {
                key: key.to_string(),
                content_encoding: enc_name.to_string(),
                index: Nat::from(chunk_index + 1),
                sha256: Some(ByteBuf::from(content_sha256)),
            })
        }
    }
}

impl HttpRequest {
    pub fn get_path(&self) -> &str {
        match self.url.find('?') {
            Some(i) => &self.url[..i],
            None => &self.url[..],
        }
    }

    pub fn get_header_value(&self, header_key: &str) -> Option<&String> {
        self.headers
            .iter()
            .find_map(|(k, v)| k.eq_ignore_ascii_case(header_key).then_some(v))
    }

    pub fn redirect_from_raw_to_certified_domain(&self) -> HttpResponse {
        #[cfg(not(test))]
        let canister_id = ic_cdk::api::id().to_text();
        #[cfg(test)]
        let canister_id = self.get_canister_id();

        let location = format!("https://{canister_id}.ic0.app{path}", path = self.url);
        HttpResponse::build_redirect(HTTP_REDIRECT_PERMANENT, location)
    }

    #[cfg(test)]
    pub fn get_canister_id(&self) -> &str {
        if let Some(host_header) = self.get_header_value("Host") {
            if host_header.contains(".localhost") || host_header.contains(".app") {
                return host_header.split('.').next().unwrap();
            } else if let Some(t) = self.url.split("canisterId=").nth(1) {
                let x = t.split_once('&');
                if let Some(c) = x {
                    return c.0;
                }
            }
        }
        unreachable!()
    }

    pub fn is_raw_domain(&self) -> bool {
        if let Some(host_header) = self.get_header_value("Host") {
            host_header.contains(".raw.ic")
        } else {
            false
        }
    }
}

impl HttpResponse {
    #[allow(clippy::too_many_arguments)]
    pub fn build_ok(
        asset: &Asset,
        enc_name: &str,
        enc: &AssetEncoding,
        key: &str,
        chunk_index: usize,
        certificate_header: Option<HeaderField>,
        callback: Func,
        etags: Vec<Hash>,
    ) -> HttpResponse {
        let mut headers =
            HashMap::from([("content-type".to_string(), asset.content_type.to_string())]);
        if enc_name != "identity" {
            headers.insert("content-encoding".to_string(), enc_name.to_string());
        }
        if let Some(head) = certificate_header {
            headers.insert(head.0, head.1);
        }
        if let Some(max_age) = asset.max_age {
            headers.insert("cache-control".to_string(), format!("max-age={}", max_age));
        }
        if let Some(arg_headers) = asset.headers.as_ref() {
            for (k, v) in arg_headers {
                headers.insert(k.to_owned().to_lowercase(), v.to_owned());
            }
        }

        let streaming_strategy = StreamingCallbackToken::create_token(
            enc_name,
            enc.content_chunks.len(),
            enc.sha256,
            key,
            chunk_index,
        )
        .map(|token| StreamingStrategy::Callback { callback, token });

        let (status_code, body) = if etags.contains(&enc.sha256) {
            (304, RcBytes::default())
        } else {
            headers.insert(
                "etag".to_string(),
                format!("\"{}\"", hex::encode(enc.sha256)),
            );
            (200, enc.content_chunks[chunk_index].clone())
        };

        HttpResponse {
            status_code,
            headers: headers.into_iter().collect::<_>(),
            body,
            streaming_strategy,
        }
    }

    pub fn build_400(err_msg: &str) -> Self {
        HttpResponse {
            status_code: 400,
            headers: vec![],
            body: RcBytes::from(ByteBuf::from(err_msg)),
            streaming_strategy: None,
        }
    }

    pub fn build_404(certificate_header: HeaderField) -> HttpResponse {
        HttpResponse {
            status_code: 404,
            headers: vec![certificate_header],
            body: RcBytes::from(ByteBuf::from("not found")),
            streaming_strategy: None,
        }
    }

    pub fn build_redirect(status_code: u16, location: String) -> HttpResponse {
        HttpResponse {
            status_code,
            headers: vec![("Location".to_string(), location)],
            body: RcBytes::from(ByteBuf::default()),
            streaming_strategy: None,
        }
    }
}
