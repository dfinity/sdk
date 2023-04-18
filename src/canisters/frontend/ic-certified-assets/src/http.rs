use crate::certification_types::CertificateExpression;
use crate::rc_bytes::RcBytes;
use crate::state_machine::{encoding_certification_order, Asset, AssetEncoding};
use candid::{CandidType, Deserialize, Func, Nat};
use ic_certified_map::{Hash, HashTree};
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::Digest;

const HTTP_REDIRECT_PERMANENT: u16 = 308;

pub const IC_CERTIFICATE_EXPRESSION_VALUE: &str = r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type"{headers}]}}}})"#;

pub type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub certificate_version: Option<u16>,
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

    // Spec:
    // If not set: assume version 1.
    // If available: use requested certificate version.
    // If requested version is not available: use latest available version.
    pub fn get_certificate_version(&self) -> u16 {
        if self.certificate_version.is_none() || self.certificate_version == Some(1) {
            1
        } else {
            2 // latest available
        }
    }

    pub fn redirect_from_raw_to_certified_domain(&self) -> HttpResponse {
        #[cfg(not(test))]
        let canister_id = ic_cdk::api::id().to_text();
        #[cfg(test)]
        let canister_id = self.get_canister_id();

        let location = format!("https://{canister_id}.icp0.io{path}", path = self.url);
        HttpResponse::build_redirect(HTTP_REDIRECT_PERMANENT, location)
    }

    #[cfg(test)]
    pub fn get_canister_id(&self) -> &str {
        if let Some(host_header) = self.get_header_value("Host") {
            if host_header.contains(".localhost") || host_header.contains(".io") {
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
        certificate_header: Option<&HeaderField>,
        callback: &Func,
        etags: &[Hash],
        cert_version: u16,
    ) -> HttpResponse {
        let mut headers = asset.get_headers_for_asset(enc_name, cert_version);
        if let Some(head) = certificate_header {
            headers.insert(head.0.clone(), head.1.clone());
        }

        let streaming_strategy = StreamingCallbackToken::create_token(
            enc_name,
            enc.content_chunks.len(),
            enc.sha256,
            key,
            chunk_index,
        )
        .map(|token| StreamingStrategy::Callback {
            callback: callback.clone(),
            token,
        });

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

    #[allow(clippy::too_many_arguments)]
    pub fn build_ok_from_requested_encodings(
        asset: &Asset,
        requested_encodings: &[String],
        key: &str,
        chunk_index: usize,
        certificate_header: Option<&HeaderField>,
        callback: &Func,
        etags: &[Hash],
        cert_version: u16,
    ) -> Option<HttpResponse> {
        let most_important_v1 = asset.most_important_encoding_v1();

        // Return a requested encoding that is certified
        for enc_name in requested_encodings.iter() {
            if let Some(enc) = asset.encodings.get(enc_name) {
                if enc.certified {
                    if cert_version == 1 {
                        // In v1, only the most important encoding is certified.
                        if enc_name != &most_important_v1 {
                            continue;
                        }
                    }
                    return Some(Self::build_ok(
                        asset,
                        enc_name,
                        enc,
                        key,
                        chunk_index,
                        certificate_header,
                        callback,
                        etags,
                        cert_version,
                    ));
                }
            }
        }

        // None of the requested encodings are available with certification
        // In v1, a first fall-back measure is to return a non-certified encoding, if a requested encoding is available
        if cert_version == 1 {
            for enc_name in requested_encodings.iter() {
                if let Some(enc) = asset.encodings.get(enc_name) {
                    return Some(Self::build_ok(
                        asset,
                        enc_name,
                        enc,
                        key,
                        chunk_index,
                        None,
                        callback,
                        etags,
                        cert_version,
                    ));
                }
            }
        }

        // None of the requested encodings are available - fall back to the best we have
        for enc_name in encoding_certification_order(asset.encodings.keys()) {
            if let Some(enc) = asset.encodings.get(&enc_name) {
                // In v1, only the most important encoding is certified.
                if enc_name != most_important_v1 {
                    continue;
                }
                if enc.certified {
                    return Some(Self::build_ok(
                        asset,
                        &enc_name,
                        enc,
                        key,
                        chunk_index,
                        certificate_header,
                        callback,
                        etags,
                        cert_version,
                    ));
                }
            }
        }
        None
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

pub fn build_ic_certificate_expression_from_headers_and_encoding(
    header_names: &[&str],
    encoding_name: &str,
) -> CertificateExpression {
    let mut headers = header_names
        .iter()
        .map(|h| format!(", \"{}\"", h))
        .collect::<Vec<_>>()
        .join("");
    if encoding_name != "identity" {
        headers = format!(", \"content-encoding\"{}", headers);
    }

    let expression = IC_CERTIFICATE_EXPRESSION_VALUE.replace("{headers}", &headers);
    let hash = sha2::Sha256::digest(expression.as_bytes())
        .into_iter()
        .collect();
    CertificateExpression { expression, hash }
}

pub fn witness_to_header_v1(witness: HashTree, certificate: &[u8]) -> HeaderField {
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    witness.serialize(&mut serializer).unwrap();
    (
        "IC-Certificate".to_string(),
        String::from("certificate=:")
            + &base64::encode(certificate)
            + ":, tree=:"
            + &base64::encode(&serializer.into_inner())
            + ":",
    )
}

pub fn witness_to_header_v2(witness: HashTree, certificate: &[u8], expr_path: &str) -> HeaderField {
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    witness.serialize(&mut serializer).unwrap();

    (
        "IC-Certificate".to_string(),
        String::from("version=2, ")
            + "certificate=:"
            + &base64::encode(certificate)
            + ":, tree=:"
            + &base64::encode(&serializer.into_inner())
            + ":, expr_path=:"
            + expr_path
            + ":",
    )
}
