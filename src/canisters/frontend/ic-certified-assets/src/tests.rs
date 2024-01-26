use crate::asset_certification::types::http::{
    CallbackFunc, HttpRequest, HttpResponse, StreamingCallbackToken, StreamingStrategy,
};
use crate::state_machine::{StableState, State, BATCH_EXPIRY_NANOS};
use crate::types::{
    AssetProperties, BatchId, BatchOperation, CommitBatchArguments, CommitProposedBatchArguments,
    ComputeEvidenceArguments, CreateAssetArguments, CreateChunkArg, DeleteAssetArguments,
    DeleteBatchArguments, GetArg, GetChunkArg, SetAssetContentArguments,
    SetAssetPropertiesArguments,
};
use crate::url_decode::{url_decode, UrlDecodeError};
use candid::{Nat, Principal};
use ic_certification_testing::CertificateBuilder;
use ic_crypto_tree_hash::Digest;
use ic_response_verification_test_utils::{
    base64_encode, create_canister_id, get_current_timestamp,
};
use serde_bytes::ByteBuf;
use std::collections::HashMap;

// from ic-response-verification tests
const MAX_CERT_TIME_OFFSET_NS: u128 = 300_000_000_000;

fn some_principal() -> Principal {
    Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()
}

fn unused_callback() -> CallbackFunc {
    CallbackFunc::new(some_principal(), "unused".to_string())
}

pub fn verify_response(
    state: &State,
    request: &HttpRequest,
    response: &HttpResponse,
) -> anyhow::Result<bool> {
    let mut response = response.clone();
    let current_time = get_current_timestamp();
    let canister_id = create_canister_id("rdmx6-jaaaa-aaaaa-aaadq-cai");
    let min_requested_verification_version = request.get_certificate_version();

    // inject certificate into IC-Certificate header with 'certificate=::'
    let data = CertificateBuilder::new(
        &canister_id.to_string(),
        Digest(state.root_hash()).as_bytes(),
    )?
    .with_time(current_time)
    .build()?;
    let replacement_cert_value = base64_encode(&data.cbor_encoded_certificate);
    let (_, header_value) = response
        .headers
        .iter_mut()
        .find(|(header, _)| header == "IC-Certificate")
        .expect("HttpResponse is missing 'IC-Certificate' header");
    *header_value = header_value.replace(
        "certificate=::",
        &format!("certificate=:{replacement_cert_value}:"),
    );

    // actual verification
    let request = ic_http_certification::http::HttpRequest {
        method: request.method.clone(),
        url: request.url.clone(),
        headers: request.headers.clone(),
        body: request.body[..].into(),
    };
    let response = ic_http_certification::http::HttpResponse {
        status_code: response.status_code,
        headers: response.headers,
        body: response.body[..].into(),
        upgrade: None,
    };
    Ok(ic_response_verification::verify_request_response_pair(
        request,
        response,
        canister_id.as_ref(),
        current_time,
        MAX_CERT_TIME_OFFSET_NS,
        &data.root_key,
        min_requested_verification_version.try_into().unwrap(),
    )
    .map(|res| res.response.is_some())?)
}

fn certified_http_request(state: &State, request: HttpRequest) -> HttpResponse {
    let response = state.http_request(request.clone(), &[], unused_callback());
    match verify_response(state, &request, &response) {
        Err(err) => panic!(
            "Response verification failed with error {:?}. Response: {:#?}",
            err, response
        ),
        Ok(success) => {
            if !success {
                panic!("Response verification failed. Response: {:?}", response)
            }
        }
    }
    response
}

struct AssetBuilder {
    name: String,
    content_type: String,
    encodings: Vec<(String, Vec<ByteBuf>)>,
    max_age: Option<u64>,
    headers: Option<HashMap<String, String>>,
    aliasing: Option<bool>,
    allow_raw_access: Option<bool>,
}

impl AssetBuilder {
    fn new(name: impl AsRef<str>, content_type: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            content_type: content_type.as_ref().to_string(),
            encodings: vec![],
            max_age: None,
            headers: None,
            aliasing: None,
            allow_raw_access: None,
        }
    }

    fn with_max_age(mut self, max_age: u64) -> Self {
        self.max_age = Some(max_age);
        self
    }

    fn with_encoding(mut self, name: impl AsRef<str>, chunks: Vec<impl AsRef<[u8]>>) -> Self {
        self.encodings.push((
            name.as_ref().to_string(),
            chunks
                .into_iter()
                .map(|c| ByteBuf::from(c.as_ref().to_vec()))
                .collect(),
        ));
        self
    }

    fn with_header(mut self, header_key: &str, header_value: &str) -> Self {
        let hm = self.headers.get_or_insert(HashMap::new());
        hm.insert(header_key.to_string(), header_value.to_string());
        self
    }

    fn with_aliasing(mut self, aliasing: bool) -> Self {
        self.aliasing = Some(aliasing);
        self
    }

    fn with_allow_raw_access(mut self, allow_raw_access: Option<bool>) -> Self {
        self.allow_raw_access = allow_raw_access;
        self
    }
}

struct RequestBuilder {
    resource: String,
    method: String,
    headers: Vec<(String, String)>,
    body: ByteBuf,
    certificate_version: Option<u16>,
}

impl RequestBuilder {
    fn get(resource: impl AsRef<str>) -> Self {
        Self {
            resource: resource.as_ref().to_string(),
            method: "GET".to_string(),
            headers: vec![],
            body: ByteBuf::new(),
            certificate_version: None,
        }
    }

    fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers
            .push((name.as_ref().to_string(), value.as_ref().to_string()));
        self
    }

    fn with_certificate_version(mut self, version: u16) -> Self {
        self.certificate_version = Some(version);
        self
    }

    fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.resource,
            headers: self.headers,
            body: self.body,
            certificate_version: self.certificate_version,
        }
    }
}

fn create_assets(state: &mut State, time_now: u64, assets: Vec<AssetBuilder>) -> BatchId {
    let batch_id = state.create_batch(time_now).unwrap();

    let operations =
        assemble_create_assets_and_set_contents_operations(state, time_now, assets, &batch_id);

    state
        .commit_batch(
            CommitBatchArguments {
                batch_id: batch_id.clone(),
                operations,
            },
            time_now,
        )
        .unwrap();

    batch_id
}

fn create_assets_by_proposal(
    state: &mut State,
    time_now: u64,
    assets: Vec<AssetBuilder>,
) -> BatchId {
    let batch_id = state.create_batch(time_now).unwrap();

    let operations =
        assemble_create_assets_and_set_contents_operations(state, time_now, assets, &batch_id);

    state
        .propose_commit_batch(CommitBatchArguments {
            batch_id: batch_id.clone(),
            operations,
        })
        .unwrap();

    let evidence = state
        .compute_evidence(ComputeEvidenceArguments {
            batch_id: batch_id.clone(),
            max_iterations: Some(100),
        })
        .unwrap()
        .unwrap();

    state
        .commit_proposed_batch(
            CommitProposedBatchArguments {
                batch_id: batch_id.clone(),
                evidence,
            },
            time_now,
        )
        .unwrap();

    batch_id
}

fn assemble_create_assets_and_set_contents_operations(
    state: &mut State,
    time_now: u64,
    assets: Vec<AssetBuilder>,
    batch_id: &BatchId,
) -> Vec<BatchOperation> {
    let mut operations = vec![];

    for asset in assets {
        if state.get_asset_properties(asset.name.clone()).is_ok() {
            operations.push(BatchOperation::DeleteAsset(DeleteAssetArguments {
                key: asset.name.clone(),
            }));
        }
        operations.push(BatchOperation::CreateAsset(CreateAssetArguments {
            key: asset.name.clone(),
            content_type: asset.content_type,
            max_age: asset.max_age,
            headers: asset.headers,
            enable_aliasing: asset.aliasing,
            allow_raw_access: asset.allow_raw_access,
        }));

        for (enc, chunks) in asset.encodings {
            let mut chunk_ids = vec![];
            for chunk in chunks {
                chunk_ids.push(
                    state
                        .create_chunk(
                            CreateChunkArg {
                                batch_id: batch_id.clone(),
                                content: chunk,
                            },
                            time_now,
                        )
                        .unwrap(),
                );
            }

            operations.push(BatchOperation::SetAssetContent({
                SetAssetContentArguments {
                    key: asset.name.clone(),
                    content_encoding: enc,
                    chunk_ids,
                    sha256: None,
                }
            }));
        }
    }
    operations
}

fn delete_batch(state: &mut State, batch_id: BatchId) {
    state
        .delete_batch(DeleteBatchArguments { batch_id })
        .unwrap();
}

fn lookup_header<'a>(response: &'a HttpResponse, header: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find_map(|(h, v)| h.eq_ignore_ascii_case(header).then_some(v.as_str()))
}

impl State {
    fn fake_http_request(&self, host: &str, path: &str) -> HttpResponse {
        let fake_cert = [0xca, 0xfe];
        self.http_request(
            RequestBuilder::get(path).with_header("Host", host).build(),
            &fake_cert,
            unused_callback(),
        )
    }

    fn create_test_asset(&mut self, asset: AssetBuilder) {
        create_assets(self, 100_000_000_000, vec![asset]);
    }
}

#[test]
fn can_create_assets_using_batch_api() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let batch_id = create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents.html", "text/html").with_encoding("identity", vec![BODY])],
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);

    // Try to update a completed batch.
    let error_msg = state
        .create_chunk(
            CreateChunkArg {
                batch_id,
                content: ByteBuf::new(),
            },
            time_now,
        )
        .unwrap_err();

    let expected = "batch not found";
    assert!(
        error_msg.contains(expected),
        "expected '{}' error, got: {}",
        expected,
        error_msg
    );
}

#[test]
fn serve_correct_encoding_v1() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const IDENTITY_BODY: &[u8] = b"<!DOCTYPE html><html></html>";
    const GZIP_BODY: &[u8] = b"this is 'gzipped' content";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![IDENTITY_BODY])
                .with_encoding("gzip", vec![GZIP_BODY]),
            AssetBuilder::new("/only-identity.html", "text/html")
                .with_encoding("identity", vec![IDENTITY_BODY]),
            AssetBuilder::new("/no-encoding.html", "text/html"),
        ],
    );

    // Most important encoding is returned with certificate
    let identity_response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "identity")
            .build(),
    );
    assert_eq!(identity_response.status_code, 200);
    assert_eq!(identity_response.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&identity_response, "IC-Certificate").is_some());

    // If only uncertified encoding is accepted, return it without any certificate
    let gzip_response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip")
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(gzip_response.status_code, 200);
    assert_eq!(gzip_response.body.as_ref(), GZIP_BODY);
    assert!(lookup_header(&gzip_response, "IC-Certificate").is_some());

    // If no encoding matches, return most important encoding with certificate
    let unknown_encoding_response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "unknown")
            .build(),
    );
    assert_eq!(unknown_encoding_response.status_code, 200);
    assert_eq!(unknown_encoding_response.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&unknown_encoding_response, "IC-Certificate").is_some());

    let unknown_encoding_response_2 = certified_http_request(
        &state,
        RequestBuilder::get("/only-identity.html")
            .with_header("Accept-Encoding", "gzip")
            .build(),
    );
    assert_eq!(unknown_encoding_response_2.status_code, 200);
    assert_eq!(unknown_encoding_response_2.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&unknown_encoding_response_2, "IC-Certificate").is_some());

    // Serve 404 if the requested asset has no encoding uploaded at all
    // certification v1 cannot certify 404
    let no_encoding_response = state.http_request(
        RequestBuilder::get("/no-encoding.html")
            .with_header("Accept-Encoding", "identity")
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(no_encoding_response.status_code, 404);
    assert_eq!(no_encoding_response.body.as_ref(), "not found".as_bytes());
}

#[test]
fn serve_correct_encoding_v2() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const IDENTITY_BODY: &[u8] = b"<!DOCTYPE html><html></html>";
    const GZIP_BODY: &[u8] = b"this is 'gzipped' content";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![IDENTITY_BODY])
                .with_encoding("gzip", vec![GZIP_BODY]),
            AssetBuilder::new("/no-encoding.html", "text/html"),
        ],
    );

    let identity_response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
    );
    assert_eq!(identity_response.status_code, 200);
    assert_eq!(identity_response.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&identity_response, "IC-Certificate").is_some());

    let gzip_response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip")
            .with_certificate_version(2)
            .build(),
    );
    assert_eq!(gzip_response.status_code, 200);
    assert_eq!(gzip_response.body.as_ref(), GZIP_BODY);
    assert!(lookup_header(&gzip_response, "IC-Certificate").is_some());

    let no_encoding_response = certified_http_request(
        &state,
        RequestBuilder::get("/no-encoding.html")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
    );
    assert_eq!(no_encoding_response.status_code, 404);
    assert_eq!(no_encoding_response.body.as_ref(), "not found".as_bytes());
    assert!(lookup_header(&no_encoding_response, "IC-Certificate").is_some());
}

#[test]
fn serve_fallback_v2() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html></html>";
    const OTHER_BODY: &[u8] = b"<!DOCTYPE html><html>other content</html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/index.html", "text/html")
                .with_encoding("identity", vec![INDEX_BODY]),
            AssetBuilder::new("/deep/nested/folder/index.html", "text/html")
                .with_encoding("identity", vec![OTHER_BODY]),
            AssetBuilder::new("/deep/nested/folder/a_file.html", "text/html")
                .with_encoding("identity", vec![OTHER_BODY]),
            AssetBuilder::new("/deep/nested/sibling/another_file.html", "text/html")
                .with_encoding("identity", vec![OTHER_BODY]),
            AssetBuilder::new("/deep/nested/sibling/a_file.html", "text/html")
                .with_encoding("identity", vec![OTHER_BODY]),
        ],
    );

    let identity_response = certified_http_request(
        &state,
        RequestBuilder::get("/index.html")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
    );
    let certificate_header = lookup_header(&identity_response, "IC-Certificate").unwrap();
    println!("certificate_header: {}", certificate_header);

    assert_eq!(identity_response.status_code, 200);
    assert_eq!(identity_response.body.as_ref(), INDEX_BODY);
    assert!(certificate_header.contains("expr_path=:2dn3g2lodHRwX2V4cHJqaW5kZXguaHRtbGM8JD4=:"));

    let fallback_response = certified_http_request(
        &state,
        RequestBuilder::get("/nonexistent")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
    );
    let certificate_header = lookup_header(&fallback_response, "IC-Certificate").unwrap();
    assert_eq!(fallback_response.status_code, 200);
    assert_eq!(fallback_response.body.as_ref(), INDEX_BODY);
    assert!(certificate_header.contains("expr_path=:2dn3gmlodHRwX2V4cHJjPCo+:"));

    let valid_response = certified_http_request(
        &state,
        RequestBuilder::get("/deep/nested/folder/a_file.html")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
    );
    assert_eq!(valid_response.status_code, 200);
    assert_eq!(valid_response.body.as_ref(), OTHER_BODY);

    let fallback_response = certified_http_request(
        &state,
        RequestBuilder::get("/deep/nested/folder/nonexistent")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
    );
    assert_eq!(fallback_response.status_code, 200);
    assert_eq!(fallback_response.body.as_ref(), INDEX_BODY);
}

#[test]
fn serve_fallback_v1() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/index.html", "text/html")
            .with_encoding("identity", vec![INDEX_BODY])],
    );

    let identity_response = certified_http_request(
        &state,
        RequestBuilder::get("/index.html")
            .with_header("Accept-Encoding", "identity")
            .build(),
    );
    assert_eq!(identity_response.status_code, 200);
    assert_eq!(identity_response.body.as_ref(), INDEX_BODY);
    assert!(lookup_header(&identity_response, "IC-Certificate").is_some());

    let fallback_response = certified_http_request(
        &state,
        RequestBuilder::get("/nonexistent")
            .with_header("Accept-Encoding", "identity")
            .build(),
    );
    assert_eq!(fallback_response.status_code, 200);
    assert_eq!(fallback_response.body.as_ref(), INDEX_BODY);
    assert!(lookup_header(&fallback_response, "IC-Certificate").is_some());
}

#[test]
fn can_create_assets_using_batch_proposal_api() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let batch_id = create_assets_by_proposal(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents.html", "text/html").with_encoding("identity", vec![BODY])],
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);

    // Try to update a completed batch.
    let error_msg = state
        .create_chunk(
            CreateChunkArg {
                batch_id,
                content: ByteBuf::new(),
            },
            time_now,
        )
        .unwrap_err();

    let expected = "batch not found";
    assert!(
        error_msg.contains(expected),
        "expected '{}' error, got: {}",
        expected,
        error_msg
    );
}

#[test]
fn batches_are_dropped_after_timeout() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let _chunk_1 = state
        .create_chunk(
            CreateChunkArg {
                batch_id: batch_1.clone(),
                content: ByteBuf::from(BODY.to_vec()),
            },
            time_now,
        )
        .unwrap();

    let time_now = time_now + BATCH_EXPIRY_NANOS + 1;
    let _batch_2 = state.create_batch(time_now);

    match state.create_chunk(
        CreateChunkArg {
            batch_id: batch_1,
            content: ByteBuf::from(BODY.to_vec()),
        },
        time_now,
    ) {
        Err(err) if err.contains("batch not found") => (),
        other => panic!("expected 'batch not found' error, got: {:?}", other),
    }
}

#[test]
fn can_propose_commit_batch_exactly_once() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    let args = CommitBatchArguments {
        batch_id: batch_1,
        operations: vec![],
    };
    assert_eq!(Ok(()), state.propose_commit_batch(args.clone()));
    match state.propose_commit_batch(args) {
        Err(err) if err == *"batch already has proposed CommitBatchArguments" => {}
        other => panic!("expected batch already proposed error, got: {:?}", other),
    };
}

#[test]
fn cannot_create_chunk_in_proposed_batch_() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    let args = CommitBatchArguments {
        batch_id: batch_1.clone(),
        operations: vec![],
    };
    assert_eq!(Ok(()), state.propose_commit_batch(args));

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";
    match state.create_chunk(
        CreateChunkArg {
            batch_id: batch_1,
            content: ByteBuf::from(BODY.to_vec()),
        },
        time_now,
    ) {
        Err(err) if err == *"batch has been proposed" => {}
        other => panic!("expected batch already proposed error, got: {:?}", other),
    }
}

#[test]
fn batches_with_proposed_commit_args_do_not_expire() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let _chunk_1 = state
        .create_chunk(
            CreateChunkArg {
                batch_id: batch_1.clone(),
                content: ByteBuf::from(BODY.to_vec()),
            },
            time_now,
        )
        .unwrap();

    let args = CommitBatchArguments {
        batch_id: batch_1.clone(),
        operations: vec![],
    };
    assert_eq!(Ok(()), state.propose_commit_batch(args));

    let time_now = time_now + BATCH_EXPIRY_NANOS + 1;
    let _batch_2 = state.create_batch(time_now);

    match state.create_chunk(
        CreateChunkArg {
            batch_id: batch_1,
            content: ByteBuf::from(BODY.to_vec()),
        },
        time_now,
    ) {
        Err(err) if err.contains("batch not found") => (),
        other => panic!("expected 'batch not found' error, got: {:?}", other),
    }
}

#[test]
fn batches_with_evidence_do_not_expire() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let _chunk_1 = state
        .create_chunk(
            CreateChunkArg {
                batch_id: batch_1.clone(),
                content: ByteBuf::from(BODY.to_vec()),
            },
            time_now,
        )
        .unwrap();

    let args = CommitBatchArguments {
        batch_id: batch_1.clone(),
        operations: vec![],
    };
    assert_eq!(Ok(()), state.propose_commit_batch(args));
    assert!(matches!(
        state.compute_evidence(ComputeEvidenceArguments {
            batch_id: batch_1.clone(),
            max_iterations: Some(3),
        }),
        Ok(Some(_))
    ));

    let time_now = time_now + BATCH_EXPIRY_NANOS + 1;
    let _batch_2 = state.create_batch(time_now);

    match state.create_chunk(
        CreateChunkArg {
            batch_id: batch_1,
            content: ByteBuf::from(BODY.to_vec()),
        },
        time_now,
    ) {
        Err(err) if err == *"batch has been proposed" => {}
        other => panic!("expected batch already proposed error, got: {:?}", other),
    }
}

#[test]
fn can_delete_proposed_batch() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    let args = CommitBatchArguments {
        batch_id: batch_1.clone(),
        operations: vec![],
    };
    assert_eq!(Ok(()), state.propose_commit_batch(args));
    let delete_args = DeleteBatchArguments { batch_id: batch_1 };
    assert_eq!(Ok(()), state.delete_batch(delete_args.clone()));
    assert_eq!(
        Err("batch not found".to_string()),
        state.delete_batch(delete_args)
    );
}

#[test]
fn can_delete_batch_with_chunks() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now).unwrap();

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";
    let _chunk_1 = state
        .create_chunk(
            CreateChunkArg {
                batch_id: batch_1.clone(),
                content: ByteBuf::from(BODY.to_vec()),
            },
            time_now,
        )
        .unwrap();

    let delete_args = DeleteBatchArguments { batch_id: batch_1 };
    assert_eq!(Ok(()), state.delete_batch(delete_args.clone()));
    assert_eq!(
        Err("batch not found".to_string()),
        state.delete_batch(delete_args)
    );
}

#[test]
fn returns_index_file_for_missing_assets() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>Index</html>";
    const OTHER_BODY: &[u8] = b"<!DOCTYPE html><html>Other</html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/index.html", "text/html")
                .with_encoding("identity", vec![INDEX_BODY]),
            AssetBuilder::new("/other.html", "text/html")
                .with_encoding("identity", vec![OTHER_BODY]),
        ],
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/missing.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), INDEX_BODY);
}

#[test]
fn preserves_state_on_stable_roundtrip() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>Index</html>";

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/index.html", "text/html")
            .with_encoding("identity", vec![INDEX_BODY])],
    );

    let stable_state: StableState = state.into();
    let state: State = stable_state.into();

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/index.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), INDEX_BODY);
}

#[test]
fn uses_streaming_for_multichunk_assets() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY_CHUNK_1: &[u8] = b"<!DOCTYPE html>";
    const INDEX_BODY_CHUNK_2: &[u8] = b"<html>Index</html>";

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/index.html", "text/html")
            .with_encoding("identity", vec![INDEX_BODY_CHUNK_1, INDEX_BODY_CHUNK_2])],
    );

    let streaming_callback = CallbackFunc::new(some_principal(), "stream".to_string());
    let response = state.http_request(
        RequestBuilder::get("/index.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        streaming_callback.clone(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), INDEX_BODY_CHUNK_1);

    let StreamingStrategy::Callback { callback, token } = response
        .streaming_strategy
        .expect("missing streaming strategy");
    assert_eq!(callback, streaming_callback);

    // sha256 is required
    assert_eq!(
        state
            .http_request_streaming_callback(StreamingCallbackToken {
                key: "/index.html".to_string(),
                content_encoding: "identity".to_string(),
                index: Nat::from(1_u8),
                sha256: None,
            })
            .unwrap_err(),
        "sha256 required"
    );

    let streaming_response = state.http_request_streaming_callback(token).unwrap();
    assert_eq!(streaming_response.body.as_ref(), INDEX_BODY_CHUNK_2);
    assert!(
        streaming_response.token.is_none(),
        "Unexpected streaming response: {:?}",
        streaming_response
    );
}

#[test]
fn get_and_get_chunk_for_multichunk_assets() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY_CHUNK_0: &[u8] = b"<!DOCTYPE html>";
    const INDEX_BODY_CHUNK_1: &[u8] = b"<html>Index</html>";

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/index.html", "text/html")
            .with_encoding("identity", vec![INDEX_BODY_CHUNK_0, INDEX_BODY_CHUNK_1])],
    );

    let chunk_0 = state
        .get(GetArg {
            key: "/index.html".to_string(),
            accept_encodings: vec!["identity".to_string()],
        })
        .unwrap();
    assert_eq!(chunk_0.content.as_ref(), INDEX_BODY_CHUNK_0);

    let chunk_1 = state
        .get_chunk(GetChunkArg {
            key: "/index.html".to_string(),
            content_encoding: "identity".to_string(),
            index: Nat::from(1_u8),
            sha256: chunk_0.sha256,
        })
        .unwrap();
    assert_eq!(chunk_1.as_ref(), INDEX_BODY_CHUNK_1);

    // get_chunk fails if we don't pass the sha256
    assert_eq!(
        state
            .get_chunk(GetChunkArg {
                key: "/index.html".to_string(),
                content_encoding: "identity".to_string(),
                index: Nat::from(1_u8),
                sha256: None,
            })
            .unwrap_err(),
        "sha256 required".to_string()
    );
}

#[test]
fn supports_max_age_headers() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html").with_encoding("identity", vec![BODY]),
            AssetBuilder::new("/max-age.html", "text/html")
                .with_max_age(604800)
                .with_encoding("identity", vec![BODY]),
        ],
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);
    assert!(
        lookup_header(&response, "Cache-Control").is_none(),
        "Unexpected Cache-Control header in response: {:#?}",
        response,
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/max-age.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);
    assert_eq!(
        lookup_header(&response, "Cache-Control"),
        Some("max-age=604800"),
        "No matching Cache-Control header in response: {:#?}",
        response,
    );
}

#[test]
fn check_url_decode() {
    assert_eq!(
        url_decode("/%"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(url_decode("/%%"), Ok("/%".to_string()));
    assert_eq!(url_decode("/%20a"), Ok("/ a".to_string()));
    assert_eq!(
        url_decode("/%%+a%20+%@"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(
        url_decode("/has%percent.txt"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(url_decode("/%e6"), Ok("/æ".to_string()));
}

#[test]
fn supports_custom_http_headers() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![BODY])
                .with_header("Access-Control-Allow-Origin", "*"),
            AssetBuilder::new("/max-age.html", "text/html")
                .with_max_age(604800)
                .with_encoding("identity", vec![BODY])
                .with_header("X-Content-Type-Options", "nosniff"),
        ],
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);
    assert!(
        lookup_header(&response, "Access-Control-Allow-Origin").is_some(),
        "Missing Access-Control-Allow-Origin header in response: {:#?}",
        response,
    );
    assert!(
        lookup_header(&response, "Access-Control-Allow-Origin") == Some("*"),
        "Incorrect value for Access-Control-Allow-Origin header in response: {:#?}",
        response,
    );

    let response = certified_http_request(
        &state,
        RequestBuilder::get("/max-age.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);
    assert_eq!(
        lookup_header(&response, "Cache-Control"),
        Some("max-age=604800"),
        "No matching Cache-Control header in response: {:#?}",
        response,
    );
    assert!(
        lookup_header(&response, "X-Content-Type-Options").is_some(),
        "Missing X-Content-Type-Options header in response: {:#?}",
        response,
    );
    assert!(
        lookup_header(&response, "X-Content-Type-Options") == Some("nosniff"),
        "Incorrect value for X-Content-Type-Options header in response: {:#?}",
        response,
    );
}

#[test]
fn supports_getting_and_setting_asset_properties() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![BODY])
                .with_header("Access-Control-Allow-Origin", "*"),
            AssetBuilder::new("/max-age.html", "text/html")
                .with_max_age(604800)
                .with_encoding("identity", vec![BODY])
                .with_header("X-Content-Type-Options", "nosniff"),
        ],
    );

    assert_eq!(
        state.get_asset_properties("/contents.html".into()),
        Ok(AssetProperties {
            max_age: None,
            headers: Some(HashMap::from([(
                "Access-Control-Allow-Origin".into(),
                "*".into()
            )])),
            allow_raw_access: None,
            is_aliased: None
        })
    );
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(604800),
            headers: Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )])),
            allow_raw_access: None,
            is_aliased: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(1)),
            headers: Some(Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )]))),
            allow_raw_access: None,
            is_aliased: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )])),
            allow_raw_access: None,
            is_aliased: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(None),
            headers: Some(None),
            allow_raw_access: None,
            is_aliased: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: None,
            headers: None,
            allow_raw_access: None,
            is_aliased: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(1)),
            headers: Some(Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )]))),
            allow_raw_access: None,
            is_aliased: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )])),
            allow_raw_access: None,
            is_aliased: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: Some(Some(HashMap::from([("new-header".into(), "value".into())]))),
            allow_raw_access: None,
            is_aliased: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([("new-header".into(), "value".into())])),
            allow_raw_access: None,
            is_aliased: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(2)),
            headers: None,
            allow_raw_access: None,
            is_aliased: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: Some(HashMap::from([("new-header".into(), "value".into())])),
            allow_raw_access: None,
            is_aliased: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: None,
            allow_raw_access: None,
            is_aliased: Some(Some(false))
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: Some(HashMap::from([("new-header".into(), "value".into())])),
            allow_raw_access: None,
            is_aliased: Some(false)
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: Some(None),
            allow_raw_access: None,
            is_aliased: Some(None)
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: None,
            allow_raw_access: None,
            is_aliased: None
        })
    );
}

#[test]
fn create_asset_fails_if_asset_exists() {
    let mut state = State::default();
    let time_now = 100_000_000_000;
    const FILE_BODY: &[u8] = b"<!DOCTYPE html><html>file body</html>";

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents.html", "text/html")
            .with_encoding("identity", vec![FILE_BODY])],
    );

    assert!(
        state
            .create_asset(CreateAssetArguments {
                key: "/contents.html".to_string(),
                content_type: "text/html".to_string(),
                max_age: None,
                headers: None,
                allow_raw_access: None,
                enable_aliasing: None,
            })
            .unwrap_err()
            == "asset already exists"
    );
}

#[test]
fn support_aliases() {
    let mut state = State::default();
    let time_now = 100_000_000_000;
    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>index</html>";
    const SUBDIR_INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>subdir index</html>";
    const FILE_BODY: &[u8] = b"<!DOCTYPE html><html>file body</html>";
    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY]),
            AssetBuilder::new("/index.html", "text/html")
                .with_encoding("identity", vec![INDEX_BODY]),
            AssetBuilder::new("/subdirectory/index.html", "text/html")
                .with_encoding("identity", vec![SUBDIR_INDEX_BODY]),
        ],
    );

    let normal_request =
        certified_http_request(&state, RequestBuilder::get("/contents.html").build());
    assert_eq!(normal_request.body.as_ref(), FILE_BODY);

    let alias_add_html = certified_http_request(&state, RequestBuilder::get("/contents").build());
    assert_eq!(alias_add_html.body.as_ref(), FILE_BODY);

    let root_alias = certified_http_request(&state, RequestBuilder::get("/").build());
    assert_eq!(root_alias.body.as_ref(), INDEX_BODY);

    // cannot use certified request because this produces an invalid URL
    let empty_path_alias =
        state.http_request(RequestBuilder::get("").build(), &[], unused_callback());
    assert_eq!(empty_path_alias.body.as_ref(), INDEX_BODY);

    let subdirectory_index_alias =
        certified_http_request(&state, RequestBuilder::get("/subdirectory/index").build());
    assert_eq!(subdirectory_index_alias.body.as_ref(), SUBDIR_INDEX_BODY);

    let subdirectory_index_alias_2 =
        certified_http_request(&state, RequestBuilder::get("/subdirectory/").build());
    assert_eq!(subdirectory_index_alias_2.body.as_ref(), SUBDIR_INDEX_BODY);

    let subdirectory_index_alias_3 =
        certified_http_request(&state, RequestBuilder::get("/subdirectory").build());
    assert_eq!(subdirectory_index_alias_3.body.as_ref(), SUBDIR_INDEX_BODY);
}

#[test]
fn alias_enable_and_disable() {
    let mut state = State::default();
    let time_now = 100_000_000_000;
    const SUBDIR_INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>subdir index</html>";
    const FILE_BODY: &[u8] = b"<!DOCTYPE html><html>file body</html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY]),
            AssetBuilder::new("/subdirectory/index.html", "text/html")
                .with_encoding("identity", vec![SUBDIR_INDEX_BODY]),
        ],
    );

    let alias_add_html = certified_http_request(&state, RequestBuilder::get("/contents").build());
    assert_eq!(alias_add_html.body.as_ref(), FILE_BODY);

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/contents.html".into(),
            max_age: None,
            headers: None,
            allow_raw_access: None,
            is_aliased: Some(Some(false)),
        })
        .is_ok());

    let no_more_alias = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
    assert_ne!(no_more_alias.body.as_ref(), FILE_BODY);

    let other_alias_still_works =
        certified_http_request(&state, RequestBuilder::get("/subdirectory/index").build());
    assert_eq!(other_alias_still_works.body.as_ref(), SUBDIR_INDEX_BODY);

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents.html", "text/html")
            .with_encoding("identity", vec![FILE_BODY])
            .with_aliasing(true)],
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/contents.html".into(),
            max_age: None,
            headers: None,
            allow_raw_access: None,
            is_aliased: Some(Some(true)),
        })
        .is_ok());
    let alias_add_html_again =
        certified_http_request(&state, RequestBuilder::get("/contents").build());
    assert_eq!(alias_add_html_again.body.as_ref(), FILE_BODY);
}

#[test]
fn alias_behavior_persists_through_upgrade() {
    let mut state = State::default();
    let time_now = 100_000_000_000;
    const SUBDIR_INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>subdir index</html>";
    const FILE_BODY: &[u8] = b"<!DOCTYPE html><html>file body</html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY])
                .with_aliasing(false),
            AssetBuilder::new("/subdirectory/index.html", "text/html")
                .with_encoding("identity", vec![SUBDIR_INDEX_BODY]),
        ],
    );

    let alias_disabled = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
    assert_ne!(alias_disabled.body.as_ref(), FILE_BODY);

    let alias_for_other_asset_still_works =
        certified_http_request(&state, RequestBuilder::get("/subdirectory").build());
    assert_eq!(
        alias_for_other_asset_still_works.body.as_ref(),
        SUBDIR_INDEX_BODY
    );

    let stable_state: StableState = state.into();
    let state: State = stable_state.into();

    let alias_stays_turned_off = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
    assert_ne!(alias_stays_turned_off.body.as_ref(), FILE_BODY);

    let alias_for_other_asset_still_works =
        certified_http_request(&state, RequestBuilder::get("/subdirectory").build());
    assert_eq!(
        alias_for_other_asset_still_works.body.as_ref(),
        SUBDIR_INDEX_BODY
    );
}

#[test]
fn aliasing_name_clash() {
    let mut state = State::default();
    let time_now = 100_000_000_000;
    const FILE_BODY: &[u8] = b"<!DOCTYPE html><html>file body</html>";
    const FILE_BODY_2: &[u8] = b"<!DOCTYPE html><html>second body</html>";

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents.html", "text/html")
            .with_encoding("identity", vec![FILE_BODY])],
    );

    let alias_add_html = certified_http_request(&state, RequestBuilder::get("/contents").build());
    assert_eq!(alias_add_html.body.as_ref(), FILE_BODY);

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents", "text/html")
            .with_encoding("identity", vec![FILE_BODY_2])],
    );

    let alias_doesnt_overwrite_actual_file =
        certified_http_request(&state, RequestBuilder::get("/contents").build());
    assert_eq!(
        alias_doesnt_overwrite_actual_file.body.as_ref(),
        FILE_BODY_2
    );

    state.delete_asset(DeleteAssetArguments {
        key: "/contents".to_string(),
    });

    let alias_accessible_again =
        certified_http_request(&state, RequestBuilder::get("/contents").build());
    assert_eq!(alias_accessible_again.body.as_ref(), FILE_BODY);
}

#[cfg(test)]
mod allow_raw_access {
    use super::*;

    const FILE_BODY: &[u8] = b"<!DOCTYPE html><html>file body</html>";

    #[test]
    fn redirects_from_raw_to_certified() {
        let mut state = State::default();

        state.create_test_asset(
            AssetBuilder::new("/page.html", "text/html").with_allow_raw_access(Some(false)),
        );
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/page");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.icp0.io/page"
        );
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/page");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.ic0.app/page"
        );

        state.create_test_asset(
            AssetBuilder::new("/page2.html", "text/html").with_allow_raw_access(Some(false)),
        );
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/page2");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.icp0.io/page2"
        );

        state.create_test_asset(
            AssetBuilder::new("/index.html", "text/html").with_allow_raw_access(Some(false)),
        );
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.icp0.io/"
        );

        let mut state = State::default();
        state.create_test_asset(
            AssetBuilder::new("/index.html", "text/html").with_allow_raw_access(Some(false)),
        );
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.icp0.io/"
        );
    }

    #[test]
    fn wont_redirect_from_raw_to_certified() {
        let mut state = State::default();
        state.create_test_asset(
            AssetBuilder::new("/blog.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY])
                .with_allow_raw_access(Some(true)),
        );
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/blog.html");
        dbg!(&response);
        assert_eq!(response.status_code, 200);

        let mut state = State::default();
        state.create_test_asset(
            AssetBuilder::new("/index.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY])
                .with_allow_raw_access(Some(true)),
        );
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/index.html");
        dbg!(&response);
        assert_eq!(response.status_code, 200);

        let mut state = State::default();
        state.create_test_asset(
            AssetBuilder::new("/index.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY])
                .with_allow_raw_access(Some(true)),
        );
        let response = state.fake_http_request("a-b-c.localhost:4444", "/index.html");
        dbg!(&response);
        assert_eq!(response.status_code, 200);
    }
}

#[cfg(test)]
mod certificate_expression {
    use super::*;
    use crate::asset_certification::types::http::build_ic_certificate_expression_from_headers_and_encoding;
    use ic_representation_independent_hash::Value;

    #[test]
    fn ic_certificate_expression_value_from_headers() {
        let h = [
            ("a".into(), Value::String("".into())),
            ("b".into(), Value::String("".into())),
            ("c".into(), Value::String("".into())),
        ]
        .to_vec();
        let c = build_ic_certificate_expression_from_headers_and_encoding(&h, Some("not identity"));
        assert_eq!(
            c.expression,
            r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type", "content-encoding", "a", "b", "c"]}}}})"#
        );
        let c2 = build_ic_certificate_expression_from_headers_and_encoding(&h, Some("identity"));
        assert_eq!(
            c2.expression,
            r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type", "a", "b", "c"]}}}})"#
        );
    }

    #[test]
    fn ic_certificate_expression_present_for_new_assets() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

        create_assets(
            &mut state,
            time_now,
            vec![AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![BODY])
                .with_max_age(604800)
                .with_header("Access-Control-Allow-Origin", "*")],
        );

        let v1_response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .build(),
        );

        assert!(
            lookup_header(&v1_response, "ic-certificateexpression").is_none(),
            "superfluous ic-certificateexpression header detected in cert v1"
        );

        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
        );

        assert!(
            lookup_header(&response, "ic-certificateexpression").is_some(),
            "Missing ic-certifiedexpression header in response: {:#?}",
            response,
        );
        assert_eq!(
            lookup_header(&response, "ic-certificateexpression").unwrap(),
            r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type", "cache-control", "Access-Control-Allow-Origin"]}}}})"#,
            "Missing ic-certifiedexpression header in response: {:#?}",
            response,
        );
    }

    #[test]
    fn ic_certificate_expression_gets_updated_on_asset_properties_update() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

        create_assets(
            &mut state,
            time_now,
            vec![AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("gzip", vec![BODY])
                .with_max_age(604800)
                .with_header("Access-Control-Allow-Origin", "*")],
        );

        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
        );

        assert!(
            lookup_header(&response, "ic-certificateexpression").is_some(),
            "Missing ic-certificateexpression header in response: {:#?}",
            response,
        );
        assert_eq!(
            lookup_header(&response, "ic-certificateexpression").unwrap(),
            r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type", "content-encoding", "cache-control", "Access-Control-Allow-Origin"]}}}})"#,
            "Missing ic-certificateexpression header in response: {:#?}",
            response,
        );

        state
            .set_asset_properties(SetAssetPropertiesArguments {
                key: "/contents.html".into(),
                max_age: Some(None),
                headers: Some(Some(HashMap::from([(
                    "custom-header".into(),
                    "value".into(),
                )]))),
                allow_raw_access: None,
                is_aliased: None,
            })
            .unwrap();
        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
        );
        assert!(
            lookup_header(&response, "ic-certificateexpression").is_some(),
            "Missing ic-certificateexpression header in response: {:#?}",
            response,
        );
        assert_eq!(
            lookup_header(&response, "ic-certificateexpression").unwrap(),
            r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type", "content-encoding", "custom-header"]}}}})"#,
            "Missing ic-certifiedexpression header in response: {:#?}",
            response,
        );
    }
}

#[cfg(test)]
mod certification_v2 {
    use super::*;

    #[test]
    fn proper_header_structure() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        const BODY: &[u8] = b"<!DOCTYPE html><html></html>";
        const UPDATED_BODY: &[u8] = b"<!DOCTYPE html><html>lots of content!</html>";

        create_assets(
            &mut state,
            time_now,
            vec![AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![BODY])
                .with_max_age(604800)
                .with_header("Access-Control-Allow-Origin", "*")],
        );

        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
        );

        let cert_header =
            lookup_header(&response, "ic-certificate").expect("ic-certificate header missing");

        assert!(
            cert_header.contains("version=2"),
            "cert is missing version indicator or has wrong version",
        );
        assert!(cert_header.contains("certificate=:"), "cert is missing",);
        assert!(cert_header.contains("tree=:"), "tree is missing",);
        assert!(!cert_header.contains("tree=::"), "tree is empty",);
        assert!(cert_header.contains("expr_path=:"), "expr_path is missing",);
        assert!(!cert_header.contains("expr_path=::"), "expr_path is empty",);

        create_assets(
            &mut state,
            time_now,
            vec![AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![UPDATED_BODY])
                .with_max_age(604800)
                .with_header("Access-Control-Allow-Origin", "*")],
        );

        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
        );

        assert!(lookup_header(&response, "ic-certificate").is_some());
    }

    #[test]
    fn etag() {
        // For now only checks that defining a custom etag doesn't break certification.
        // Serving HTTP 304 responses if the etag matches is part of https://dfinity.atlassian.net/browse/SDK-191

        let mut state = State::default();
        let time_now = 100_000_000_000;

        const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

        create_assets(
            &mut state,
            time_now,
            vec![AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![BODY])
                .with_header("etag", "my-etag")],
        );

        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(1)
                .build(),
        );
        assert_eq!(
            lookup_header(&response, "etag").expect("ic-certificate header missing"),
            "my-etag"
        );

        let response = certified_http_request(
            &state,
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
        );
        assert_eq!(
            lookup_header(&response, "etag").expect("ic-certificate header missing"),
            "my-etag"
        );
    }
}

#[cfg(test)]
mod evidence_computation {
    use super::*;
    use crate::types::BatchOperation::SetAssetContent;
    use crate::types::{ClearArguments, ComputeEvidenceArguments, UnsetAssetContentArguments};

    #[test]
    fn evidence_with_set_single_chunk_asset_content() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        const BODY: &[u8] = b"<!DOCTYPE html><html></html>";
        let chunk_1 = state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_1.clone(),
                    content: ByteBuf::from(BODY.to_vec()),
                },
                time_now,
            )
            .unwrap();

        let create_asset = CreateAssetArguments {
            key: "/a/b/c".to_string(),
            content_type: "text/plain".to_string(),
            max_age: None,
            headers: None,
            enable_aliasing: None,
            allow_raw_access: None,
        };
        let set_asset_content = SetAssetContentArguments {
            key: "/a/b/c".to_string(),
            content_encoding: "identity".to_string(),
            chunk_ids: vec![chunk_1],
            sha256: None,
        };
        let cba = CommitBatchArguments {
            batch_id: batch_1.clone(),
            operations: vec![
                BatchOperation::CreateAsset(create_asset),
                BatchOperation::SetAssetContent(set_asset_content),
            ],
        };
        assert!(state.propose_commit_batch(cba).is_ok());
        assert!(matches!(
            state.compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            }),
            Ok(None)
        ));
        assert!(matches!(
            state.compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1,
                max_iterations: Some(1),
            }),
            Ok(Some(_))
        ));
    }

    #[test]
    fn evidence_with_set_multiple_chunk_asset_content() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        const CHUNK_1_CONTENT: &[u8] = b"<!DOCTYPE html><html></html>";
        const CHUNK_2_CONTENT: &[u8] = b"there is more content here";
        let chunk_1 = state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_1.clone(),
                    content: ByteBuf::from(CHUNK_1_CONTENT.to_vec()),
                },
                time_now,
            )
            .unwrap();
        let chunk_2 = state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_1.clone(),
                    content: ByteBuf::from(CHUNK_2_CONTENT.to_vec()),
                },
                time_now,
            )
            .unwrap();

        let create_asset = CreateAssetArguments {
            key: "/a/b/c".to_string(),
            content_type: "text/plain".to_string(),
            max_age: None,
            headers: None,
            enable_aliasing: None,
            allow_raw_access: None,
        };
        let set_asset_content = SetAssetContentArguments {
            key: "/a/b/c".to_string(),
            content_encoding: "identity".to_string(),
            chunk_ids: vec![chunk_1, chunk_2],
            sha256: None,
        };
        let cba = CommitBatchArguments {
            batch_id: batch_1.clone(),
            operations: vec![
                BatchOperation::CreateAsset(create_asset),
                BatchOperation::SetAssetContent(set_asset_content),
            ],
        };
        assert!(state.propose_commit_batch(cba).is_ok());
        assert!(matches!(
            state.compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(4),
            }),
            Ok(None)
        ));
        assert!(matches!(
            state.compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1,
                max_iterations: Some(1),
            }),
            Ok(Some(_))
        ));
    }

    #[test]
    fn evidence_with_create_asset() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_id = state.create_batch(time_now).unwrap();
        let create_asset = CreateAssetArguments {
            key: "/a/b/c".to_string(),
            content_type: "text/plain".to_string(),
            max_age: None,
            headers: None,
            enable_aliasing: None,
            allow_raw_access: None,
        };
        let cba = CommitBatchArguments {
            batch_id: batch_id.clone(),
            operations: vec![BatchOperation::CreateAsset(create_asset)],
        };

        assert!(state.propose_commit_batch(cba).is_ok());

        let compute_args = ComputeEvidenceArguments {
            batch_id,
            max_iterations: Some(1),
        };
        assert!(state
            .compute_evidence(compute_args.clone())
            .unwrap()
            .is_none());
        assert!(state.compute_evidence(compute_args).unwrap().is_some());
    }

    #[test]
    fn evidence_with_set_empty_asset_content() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_id = state.create_batch(time_now).unwrap();
        let create_asset = CreateAssetArguments {
            key: "/a/b/c".to_string(),
            content_type: "text/plain".to_string(),
            max_age: None,
            headers: None,
            enable_aliasing: None,
            allow_raw_access: None,
        };
        let set_asset_content = SetAssetContentArguments {
            key: "/a/b/c".to_string(),
            content_encoding: "identity".to_string(),
            chunk_ids: vec![],
            sha256: None,
        };
        let cba = CommitBatchArguments {
            batch_id: batch_id.clone(),
            operations: vec![
                BatchOperation::CreateAsset(create_asset),
                BatchOperation::SetAssetContent(set_asset_content),
            ],
        };
        assert!(state.propose_commit_batch(cba).is_ok());

        assert!(state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_id.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .is_none());
        assert!(state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id,
                max_iterations: Some(1),
            })
            .unwrap()
            .is_some());
    }

    #[test]
    fn evidence_with_no_operations() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_id = state.create_batch(time_now).unwrap();
        let cba = CommitBatchArguments {
            batch_id: batch_id.clone(),
            operations: vec![],
        };
        assert!(state.propose_commit_batch(cba).is_ok());

        assert!(state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id,
                max_iterations: Some(1),
            })
            .unwrap()
            .is_some());
    }

    #[test]
    fn create_asset_same_fields_same_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        {
            let batch_1 = state.create_batch(time_now).unwrap();
            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch_1.clone(),
                    operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                        key: "/a/b/c".to_string(),
                        content_type: "".to_string(),
                        max_age: None,
                        headers: None,
                        enable_aliasing: None,
                        allow_raw_access: None,
                    }),],
                })
                .is_ok());
            let evidence_1 = state
                .compute_evidence(ComputeEvidenceArguments {
                    batch_id: batch_1.clone(),
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();
            delete_batch(&mut state, batch_1);

            let batch_2 = state.create_batch(time_now).unwrap();
            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch_2.clone(),
                    operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                        key: "/a/b/c".to_string(),
                        content_type: "".to_string(),
                        max_age: None,
                        headers: None,
                        enable_aliasing: None,
                        allow_raw_access: None,
                    }),],
                })
                .is_ok());
            let evidence_2 = state
                .compute_evidence(ComputeEvidenceArguments {
                    batch_id: batch_2.clone(),
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();
            delete_batch(&mut state, batch_2);

            assert_eq!(evidence_1, evidence_2);
        }

        {
            let batch_1 = state.create_batch(time_now).unwrap();
            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch_1.clone(),
                    operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                        key: "/d".to_string(),
                        content_type: "text/plain".to_string(),
                        max_age: Some(98),
                        headers: Some(HashMap::from([
                            ("H1".to_string(), "V1".to_string()),
                            ("H2".to_string(), "V2".to_string())
                        ])),
                        enable_aliasing: Some(true),
                        allow_raw_access: Some(false),
                    }),],
                })
                .is_ok());
            let evidence_1 = state
                .compute_evidence(ComputeEvidenceArguments {
                    batch_id: batch_1.clone(),
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();
            delete_batch(&mut state, batch_1);

            let batch_2 = state.create_batch(time_now).unwrap();
            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch_2.clone(),
                    operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                        key: "/d".to_string(),
                        content_type: "text/plain".to_string(),
                        max_age: Some(98),
                        headers: Some(HashMap::from([
                            ("H1".to_string(), "V1".to_string()),
                            ("H2".to_string(), "V2".to_string())
                        ])),
                        enable_aliasing: Some(true),
                        allow_raw_access: Some(false),
                    }),],
                })
                .is_ok());
            let evidence_2 = state
                .compute_evidence(ComputeEvidenceArguments {
                    batch_id: batch_2,
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();
            assert_eq!(evidence_1, evidence_2);
        }
    }

    #[test]
    fn create_asset_arguments_key_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/a/b/c".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/d/e/f".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn create_asset_arguments_content_type_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "text/plain".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "application/octet-stream".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn create_asset_arguments_max_age_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: Some(32),
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());

        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_2);

        let batch_3 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_3.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: Some(987),
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_3 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_3,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
        assert_ne!(evidence_1, evidence_3);
        assert_ne!(evidence_2, evidence_3);
    }

    #[test]
    fn create_asset_arguments_headers_affect_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: Some(HashMap::from([("H1".to_string(), "V1".to_string()),])),
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: Some(HashMap::from([("H1".to_string(), "V2".to_string()),])),
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_2);

        let batch_3 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_3.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: Some(HashMap::from([("H2".to_string(), "V1".to_string()),])),
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());

        let evidence_3 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_3.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_3);

        let batch_4 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_4.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: Some(HashMap::from([
                        ("H1".to_string(), "V1".to_string()),
                        ("H2".to_string(), "V2".to_string()),
                    ])),
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_4 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_4,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
        assert_ne!(evidence_1, evidence_3);
        assert_ne!(evidence_1, evidence_4);
        assert_ne!(evidence_2, evidence_3);
        assert_ne!(evidence_2, evidence_4);
        assert_ne!(evidence_3, evidence_4);
    }

    #[test]
    fn create_asset_arguments_enable_aliasing_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: Some(false),
                    allow_raw_access: None,
                }),],
            })
            .is_ok());

        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_2);

        let batch_3 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_3.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: Some(true),
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_3 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_3,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
        assert_ne!(evidence_1, evidence_3);
        assert_ne!(evidence_2, evidence_3);
    }

    #[test]
    fn create_asset_arguments_allow_raw_access_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: Some(false),
                }),],
            })
            .is_ok());

        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_2);

        let batch_3 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_3.clone(),
                operations: vec![BatchOperation::CreateAsset(CreateAssetArguments {
                    key: "/".to_string(),
                    content_type: "".to_string(),
                    max_age: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: Some(true),
                }),],
            })
            .is_ok());
        let evidence_3 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_3,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
        assert_ne!(evidence_1, evidence_3);
        assert_ne!(evidence_2, evidence_3);
    }

    #[test]
    fn set_asset_content_arguments_key_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![],
                    sha256: None,
                })],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/2".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![],
                    sha256: None,
                })],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn set_asset_content_arguments_content_encoding_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![],
                    sha256: None,
                })],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "gzip".to_string(),
                    chunk_ids: vec![],
                    sha256: None,
                })],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn set_asset_content_arguments_chunk_contents_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        const CHUNK_1_CONTENT: &[u8] = b"first batch chunk content";
        const CHUNK_2_CONTENT: &[u8] = b"second batch chunk content";

        let batch_1 = state.create_batch(time_now).unwrap();
        let chunk_1 = state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_1.clone(),
                    content: ByteBuf::from(CHUNK_1_CONTENT),
                },
                time_now,
            )
            .unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![chunk_1],
                    sha256: None,
                })],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        let chunk_2 = state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_2.clone(),
                    content: ByteBuf::from(CHUNK_2_CONTENT),
                },
                time_now,
            )
            .unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![chunk_2],
                    sha256: None,
                })],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }
    #[test]
    fn set_asset_content_arguments_multiple_chunk_contents_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        const CHUNK_1_CONTENT: &[u8] = b"first chunk, same for both";
        const BATCH_1_CHUNK_2_CONTENT: &[u8] = b"first batch second chunk content";
        const BATCH_2_CHUNK_2_CONTENT: &[u8] = b"second batch second chunk content";

        let batch_1 = state.create_batch(time_now).unwrap();
        {
            let chunk_1 = state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_1.clone(),
                        content: ByteBuf::from(CHUNK_1_CONTENT),
                    },
                    time_now,
                )
                .unwrap();
            let chunk_2 = state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_1.clone(),
                        content: ByteBuf::from(BATCH_1_CHUNK_2_CONTENT),
                    },
                    time_now,
                )
                .unwrap();

            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch_1.clone(),
                    operations: vec![SetAssetContent(SetAssetContentArguments {
                        key: "/1".to_string(),
                        content_encoding: "identity".to_string(),
                        chunk_ids: vec![chunk_1, chunk_2],
                        sha256: None,
                    })],
                })
                .is_ok());
        }
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(4),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        {
            let chunk_1 = state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_2.clone(),
                        content: ByteBuf::from(CHUNK_1_CONTENT),
                    },
                    time_now,
                )
                .unwrap();
            let chunk_2 = state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_2.clone(),
                        content: ByteBuf::from(BATCH_2_CHUNK_2_CONTENT),
                    },
                    time_now,
                )
                .unwrap();
            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch_2.clone(),
                    operations: vec![SetAssetContent(SetAssetContentArguments {
                        key: "/1".to_string(),
                        content_encoding: "identity".to_string(),
                        chunk_ids: vec![chunk_1, chunk_2],
                        sha256: None,
                    })],
                })
                .is_ok());
        }
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(4),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn set_asset_content_arguments_sha256_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let sha256_1 = ByteBuf::from("01020304");
        let sha256_2 = ByteBuf::from("09080706");

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![],
                    sha256: Some(sha256_1),
                })],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![SetAssetContent(SetAssetContentArguments {
                    key: "/1".to_string(),
                    content_encoding: "identity".to_string(),
                    chunk_ids: vec![],
                    sha256: Some(sha256_2),
                })],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn unset_asset_content_arguments_key_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::UnsetAssetContent(
                    UnsetAssetContentArguments {
                        key: "/1".to_string(),
                        content_encoding: "".to_string(),
                    }
                ),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::UnsetAssetContent(
                    UnsetAssetContentArguments {
                        key: "/2".to_string(),
                        content_encoding: "".to_string(),
                    }
                ),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn unset_asset_content_arguments_content_encoding_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::UnsetAssetContent(
                    UnsetAssetContentArguments {
                        key: "/1".to_string(),
                        content_encoding: "identity".to_string(),
                    }
                ),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::UnsetAssetContent(
                    UnsetAssetContentArguments {
                        key: "/1".to_string(),
                        content_encoding: "gzip".to_string(),
                    }
                ),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn delete_asset_content_arguments_key_affects_evidence() {
        // todo
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::DeleteAsset(DeleteAssetArguments {
                    key: "/1".to_string(),
                }),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::DeleteAsset(DeleteAssetArguments {
                    key: "/2".to_string(),
                }),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn clear_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::Clear(ClearArguments {}),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![
                    BatchOperation::Clear(ClearArguments {}),
                    BatchOperation::Clear(ClearArguments {})
                ],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn set_asset_properties_arguments_key_affects_evidence() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        let batch_1 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::SetAssetProperties(
                    SetAssetPropertiesArguments {
                        key: "/1".to_string(),
                        max_age: Some(Some(100)),
                        headers: None,
                        allow_raw_access: Some(Some(false)),
                        is_aliased: Some(Some(true))
                    }
                ),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1.clone(),
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();
        delete_batch(&mut state, batch_1);

        let batch_2 = state.create_batch(time_now).unwrap();
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_2.clone(),
                operations: vec![BatchOperation::SetAssetProperties(
                    SetAssetPropertiesArguments {
                        key: "/2".to_string(),
                        max_age: Some(Some(100)),
                        headers: None,
                        allow_raw_access: Some(Some(false)),
                        is_aliased: Some(Some(true))
                    }
                ),],
            })
            .is_ok());
        let evidence_2 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        assert_ne!(evidence_1, evidence_2);
    }

    #[test]
    fn set_asset_properties_arguments_properties_affects_evidence() {
        fn generate_unique_set_asset_properties() -> Vec<SetAssetPropertiesArguments> {
            let mut result = vec![];
            for max_age in &[None, Some(None), Some(Some(100))] {
                for headers in &[
                    None,
                    Some(None),
                    Some(Some(HashMap::from([(
                        String::from("a"),
                        String::from("b"),
                    )]))),
                ] {
                    for allow_raw_access in &[None, Some(None), Some(Some(true)), Some(Some(false))]
                    {
                        for is_aliased in &[None, Some(None), Some(Some(true)), Some(Some(false))] {
                            result.push(SetAssetPropertiesArguments {
                                key: "/1".to_string(),
                                max_age: *max_age,
                                headers: headers.clone(),
                                allow_raw_access: *allow_raw_access,
                                is_aliased: *is_aliased,
                            });
                        }
                    }
                }
            }
            result
        }

        fn compute_evidence_for_set_asset_properties(
            args: SetAssetPropertiesArguments,
        ) -> serde_bytes::ByteBuf {
            let mut state = State::default();
            let time_now = 100_000_000_000;
            let batch = state.create_batch(time_now).unwrap();
            assert!(state
                .propose_commit_batch(CommitBatchArguments {
                    batch_id: batch.clone(),
                    operations: vec![BatchOperation::SetAssetProperties(args)],
                })
                .is_ok());

            state
                .compute_evidence(ComputeEvidenceArguments {
                    batch_id: batch,
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap()
        }

        let instances = generate_unique_set_asset_properties();
        let evidences = instances
            .into_iter()
            .map(compute_evidence_for_set_asset_properties)
            .collect::<Vec<_>>();

        // Check if all evidences are different.
        for i in 0..evidences.len() {
            for j in (i + 1)..evidences.len() {
                assert_ne!(evidences[i], evidences[j]);
            }
        }
    }
}

#[cfg(test)]
mod validate_commit_proposed_batch {
    use super::*;
    use crate::types::ComputeEvidenceArguments;

    #[test]
    fn batch_not_found() {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        match state.validate_commit_proposed_batch(CommitProposedBatchArguments {
            batch_id: 1_u8.into(),
            evidence: Default::default(),
        }) {
            Err(err) if err.contains("batch not found") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }

        match state.commit_proposed_batch(
            CommitProposedBatchArguments {
                batch_id: 1_u8.into(),
                evidence: Default::default(),
            },
            time_now,
        ) {
            Err(err) if err.contains("batch not found") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }
    }

    #[test]
    fn no_commit_batch_arguments() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        let batch_id = state.create_batch(time_now).unwrap();

        match state.validate_commit_proposed_batch(CommitProposedBatchArguments {
            batch_id: batch_id.clone(),
            evidence: Default::default(),
        }) {
            Err(err) if err.contains("batch does not have CommitBatchArguments") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }

        match state.commit_proposed_batch(
            CommitProposedBatchArguments {
                batch_id,
                evidence: Default::default(),
            },
            time_now,
        ) {
            Err(err) if err.contains("batch does not have CommitBatchArguments") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }
    }

    #[test]
    fn evidence_not_computed() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        let batch_id = state.create_batch(time_now).unwrap();

        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_id.clone(),
                operations: vec![],
            })
            .is_ok());

        match state.validate_commit_proposed_batch(CommitProposedBatchArguments {
            batch_id: batch_id.clone(),
            evidence: Default::default(),
        }) {
            Err(err) if err.contains("batch does not have computed evidence") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }
        match state.commit_proposed_batch(
            CommitProposedBatchArguments {
                batch_id,
                evidence: Default::default(),
            },
            time_now,
        ) {
            Err(err) if err.contains("batch does not have computed evidence") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }
    }

    #[test]
    fn evidence_does_not_match() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        let batch_id = state.create_batch(time_now).unwrap();

        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_id.clone(),
                operations: vec![],
            })
            .is_ok());

        assert!(matches!(
            state.compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_id.clone(),
                max_iterations: Some(1),
            }),
            Ok(Some(_))
        ));

        match state.validate_commit_proposed_batch(CommitProposedBatchArguments {
            batch_id: batch_id.clone(),
            evidence: Default::default(),
        }) {
            Err(err) if err.contains("does not match presented evidence") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }

        match state.commit_proposed_batch(
            CommitProposedBatchArguments {
                batch_id,
                evidence: Default::default(),
            },
            time_now,
        ) {
            Err(err) if err.contains("does not match presented evidence") => (),
            other => panic!("expected 'batch not found' error, got: {:?}", other),
        }
    }

    #[test]
    fn all_good() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        let batch_id = state.create_batch(time_now).unwrap();

        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_id.clone(),
                operations: vec![],
            })
            .is_ok());

        let compute_evidence_result = state.compute_evidence(ComputeEvidenceArguments {
            batch_id: batch_id.clone(),
            max_iterations: Some(1),
        });
        assert!(matches!(compute_evidence_result, Ok(Some(_))));

        let evidence = if let Ok(Some(computed_evidence)) = compute_evidence_result {
            computed_evidence
        } else {
            unreachable!()
        };

        assert_eq!(state.validate_commit_proposed_batch(
            CommitProposedBatchArguments {
                batch_id: batch_id.clone(),
                evidence: evidence.clone(),
            },
        ).unwrap(), "commit proposed batch 0 with evidence e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

        state
            .commit_proposed_batch(
                CommitProposedBatchArguments { batch_id, evidence },
                time_now,
            )
            .unwrap();
    }
}

#[cfg(test)]
mod configuration_methods {
    use super::*;
    use crate::types::ConfigureArguments;

    #[test]
    fn empty_config() {
        let state = State::default();

        let x = state.get_configuration();
        assert!(x.max_batches.is_none());
        assert!(x.max_chunks.is_none());
        assert!(x.max_bytes.is_none());
    }

    #[test]
    fn set_only_max_batches() {
        let mut state = State::default();

        state.configure(ConfigureArguments {
            max_batches: Some(Some(47)),
            max_chunks: None,
            max_bytes: None,
        });

        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, None);
        assert_eq!(x.max_bytes, None);
    }

    #[test]
    fn unset_only_max_batches() {
        let mut state = State::default();
        state.configure(ConfigureArguments {
            max_batches: Some(Some(47)),
            max_chunks: Some(Some(67)),
            max_bytes: Some(Some(77)),
        });
        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, Some(67));
        assert_eq!(x.max_bytes, Some(77));

        state.configure(ConfigureArguments {
            max_batches: Some(None),
            max_chunks: None,
            max_bytes: None,
        });

        let x = state.get_configuration();
        assert_eq!(x.max_batches, None);
        assert_eq!(x.max_chunks, Some(67));
        assert_eq!(x.max_bytes, Some(77));
    }

    #[test]
    fn change_only_max_batches() {
        let mut state = State::default();
        state.configure(ConfigureArguments {
            max_batches: Some(Some(47)),
            max_chunks: Some(Some(67)),
            max_bytes: Some(Some(77)),
        });
        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, Some(67));
        assert_eq!(x.max_bytes, Some(77));

        state.configure(ConfigureArguments {
            max_batches: Some(Some(35)),
            max_chunks: None,
            max_bytes: None,
        });

        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(35));
        assert_eq!(x.max_chunks, Some(67));
        assert_eq!(x.max_bytes, Some(77));
    }

    #[test]
    fn set_only_max_chunks() {
        let mut state = State::default();

        state.configure(ConfigureArguments {
            max_batches: None,
            max_chunks: Some(Some(23)),
            max_bytes: None,
        });

        let x = state.get_configuration();
        assert_eq!(x.max_batches, None);
        assert_eq!(x.max_chunks, Some(23));
        assert_eq!(x.max_bytes, None);
    }

    #[test]
    fn unset_only_max_chunks() {
        let mut state = State::default();
        state.configure(ConfigureArguments {
            max_batches: Some(Some(47)),
            max_chunks: Some(Some(67)),
            max_bytes: Some(Some(77)),
        });
        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, Some(67));
        assert_eq!(x.max_bytes, Some(77));

        state.configure(ConfigureArguments {
            max_batches: None,
            max_chunks: Some(None),
            max_bytes: None,
        });

        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, None);
        assert_eq!(x.max_bytes, Some(77));
    }

    #[test]
    fn change_only_max_chunks() {
        let mut state = State::default();
        state.configure(ConfigureArguments {
            max_batches: Some(Some(47)),
            max_chunks: Some(Some(67)),
            max_bytes: Some(Some(77)),
        });
        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, Some(67));
        assert_eq!(x.max_bytes, Some(77));

        state.configure(ConfigureArguments {
            max_batches: None,
            max_chunks: Some(Some(54)),
            max_bytes: None,
        });

        let x = state.get_configuration();
        assert_eq!(x.max_batches, Some(47));
        assert_eq!(x.max_chunks, Some(54));
        assert_eq!(x.max_bytes, Some(77));
    }
}

#[cfg(test)]
mod enforce_limits {
    use super::*;
    use crate::types::ConfigureArguments;

    #[test]
    fn cannot_create_batch_if_batch_already_proposed_with_no_batch_limit() {
        cannot_create_batch_if_batch_already_proposed_with_batch_limit(None);
    }

    #[test]
    fn cannot_create_batch_if_batch_already_proposed_with_a_batch_limit() {
        // test with a batch limit to make sure we get the right message (not: batch limit exceeded)
        cannot_create_batch_if_batch_already_proposed_with_batch_limit(Some(1));
    }

    fn cannot_create_batch_if_batch_already_proposed_with_batch_limit(max_batches: Option<u64>) {
        let mut state = State::default();
        let time_now = 100_000_000_000;

        state.configure(ConfigureArguments {
            max_batches: Some(max_batches),
            max_chunks: None,
            max_bytes: None,
        });

        let batch_id = state.create_batch(time_now).unwrap();
        let cba = CommitBatchArguments {
            batch_id: batch_id.clone(),
            operations: vec![],
        };
        assert!(state.propose_commit_batch(cba).is_ok());

        assert_eq!(state.create_batch(time_now + BATCH_EXPIRY_NANOS - 1).unwrap_err(),
                   "Batch 0 has not completed evidence computation.  Wait for it to expire or delete it to propose another.");

        assert!(state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id,
                max_iterations: Some(1),
            })
            .unwrap()
            .is_some());

        assert_eq!(
            state
                .create_batch(time_now + BATCH_EXPIRY_NANOS + 1)
                .unwrap_err(),
            "Batch 0 is already proposed.  Delete or execute it to propose another."
        );
    }

    #[test]
    fn max_batches() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        state.configure(ConfigureArguments {
            max_batches: Some(Some(3)),
            max_chunks: None,
            max_bytes: None,
        });
        state.create_batch(time_now).unwrap();
        state.create_batch(time_now).unwrap();
        state.create_batch(time_now).unwrap();
        assert_eq!(
            state.create_batch(time_now).unwrap_err(),
            "batch limit exceeded"
        );
    }

    #[test]
    fn max_chunks() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        state.configure(ConfigureArguments {
            max_batches: None,
            max_chunks: Some(Some(3)),
            max_bytes: None,
        });
        let batch_1 = state.create_batch(time_now).unwrap();
        let batch_2 = state.create_batch(time_now).unwrap();

        state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_1.clone(),
                    content: ByteBuf::new(),
                },
                time_now,
            )
            .unwrap();
        state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_2.clone(),
                    content: ByteBuf::new(),
                },
                time_now,
            )
            .unwrap();
        state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_2.clone(),
                    content: ByteBuf::new(),
                },
                time_now,
            )
            .unwrap();

        assert_eq!(
            state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_1,
                        content: ByteBuf::new(),
                    },
                    time_now
                )
                .unwrap_err(),
            "chunk limit exceeded"
        );
        assert_eq!(
            state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_2,
                        content: ByteBuf::new(),
                    },
                    time_now
                )
                .unwrap_err(),
            "chunk limit exceeded"
        );
    }

    #[test]
    fn max_bytes() {
        let mut state = State::default();
        let time_now = 100_000_000_000;
        state.configure(ConfigureArguments {
            max_batches: None,
            max_chunks: None,
            max_bytes: Some(Some(289)),
        });
        let c0 = vec![0u8; 100];
        let c1 = vec![1u8; 100];
        let c2 = vec![2u8; 90];
        let c3 = vec![3u8; 89];
        let c4 = vec![4u8; 1];

        let batch_1 = state.create_batch(time_now).unwrap();
        let batch_2 = state.create_batch(time_now).unwrap();
        state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_1.clone(),
                    content: ByteBuf::from(c0),
                },
                time_now,
            )
            .unwrap();
        state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_2.clone(),
                    content: ByteBuf::from(c1),
                },
                time_now,
            )
            .unwrap();
        assert_eq!(
            state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_2.clone(),
                        content: ByteBuf::from(c2),
                    },
                    time_now
                )
                .unwrap_err(),
            "byte limit exceeded"
        );
        state
            .create_chunk(
                CreateChunkArg {
                    batch_id: batch_2,
                    content: ByteBuf::from(c3),
                },
                time_now,
            )
            .unwrap();
        assert_eq!(
            state
                .create_chunk(
                    CreateChunkArg {
                        batch_id: batch_1,
                        content: ByteBuf::from(c4),
                    },
                    time_now
                )
                .unwrap_err(),
            "byte limit exceeded"
        );
    }
}
