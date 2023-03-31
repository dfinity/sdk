use std::collections::HashMap;

use crate::http::{HttpRequest, HttpResponse, StreamingStrategy};
use crate::state_machine::{StableState, State, BATCH_EXPIRY_NANOS};
use crate::types::{
    AssetProperties, BatchId, BatchOperation, CommitBatchArguments, CreateAssetArguments,
    CreateChunkArg, DeleteAssetArguments, DeleteBatchArguments, SetAssetContentArguments,
    SetAssetPropertiesArguments,
};
use crate::url_decode::{url_decode, UrlDecodeError};
use candid::Principal;
use serde_bytes::ByteBuf;

fn some_principal() -> Principal {
    Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()
}

fn unused_callback() -> candid::Func {
    candid::Func {
        method: "unused".to_string(),
        principal: some_principal(),
    }
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
    let batch_id = state.create_batch(time_now);

    let mut operations = vec![];

    for asset in assets {
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

    let response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
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
    let identity_response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "identity")
            .build(),
        &[],
        unused_callback(),
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
    assert!(lookup_header(&gzip_response, "IC-Certificate").is_none());

    // If no encoding matches, return most important encoding with certificate
    let unknown_encoding_response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "unknown")
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(unknown_encoding_response.status_code, 200);
    assert_eq!(unknown_encoding_response.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&unknown_encoding_response, "IC-Certificate").is_some());

    let unknown_encoding_response_2 = state.http_request(
        RequestBuilder::get("/only-identity.html")
            .with_header("Accept-Encoding", "gzip")
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(unknown_encoding_response_2.status_code, 200);
    assert_eq!(unknown_encoding_response_2.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&unknown_encoding_response_2, "IC-Certificate").is_some());

    // Serve 404 if the requested asset has no encoding uploaded at all
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
                .with_encoding("identity", vec![IDENTITY_BODY]),
            AssetBuilder::new("/contents.html", "text/html").with_encoding("gzip", vec![GZIP_BODY]),
            AssetBuilder::new("/no-encoding.html", "text/html"),
        ],
    );

    let identity_response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(identity_response.status_code, 200);
    assert_eq!(identity_response.body.as_ref(), IDENTITY_BODY);
    assert!(lookup_header(&identity_response, "IC-Certificate").is_some());

    let gzip_response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip")
            .with_certificate_version(2)
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(gzip_response.status_code, 200);
    assert_eq!(gzip_response.body.as_ref(), GZIP_BODY);
    assert!(lookup_header(&gzip_response, "IC-Certificate").is_some());

    let no_encoding_response = state.http_request(
        RequestBuilder::get("/no-encoding.html")
            .with_header("Accept-Encoding", "identity")
            .with_certificate_version(2)
            .build(),
        &[],
        unused_callback(),
    );
    assert_eq!(no_encoding_response.status_code, 404);
    assert_eq!(no_encoding_response.body.as_ref(), "not found".as_bytes());
    assert!(lookup_header(&no_encoding_response, "IC-Certificate").is_some());
}

#[test]
fn batches_are_dropped_after_timeout() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now);

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

    let batch_1 = state.create_batch(time_now);

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

    let batch_1 = state.create_batch(time_now);

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

    let batch_1 = state.create_batch(time_now);

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
        Err(err) if err == *"batch has been proposed" => {}
        other => panic!("expected batch already proposed error, got: {:?}", other),
    }
}

#[test]
fn can_delete_proposed_batch() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now);

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

    let batch_1 = state.create_batch(time_now);

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

    let response = state.http_request(
        RequestBuilder::get("/missing.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
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

    let response = state.http_request(
        RequestBuilder::get("/index.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
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

    let streaming_callback = candid::Func {
        method: "stream".to_string(),
        principal: some_principal(),
    };
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

    let streaming_response = state.http_request_streaming_callback(token).unwrap();
    assert_eq!(streaming_response.body.as_ref(), INDEX_BODY_CHUNK_2);
    assert!(
        streaming_response.token.is_none(),
        "Unexpected streaming response: {:?}",
        streaming_response
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

    let response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);
    assert!(
        lookup_header(&response, "Cache-Control").is_none(),
        "Unexpected Cache-Control header in response: {:#?}",
        response,
    );

    let response = state.http_request(
        RequestBuilder::get("/max-age.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
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

    let response = state.http_request(
        RequestBuilder::get("/contents.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
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

    let response = state.http_request(
        RequestBuilder::get("/max-age.html")
            .with_header("Accept-Encoding", "gzip,identity")
            .build(),
        &[],
        unused_callback(),
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

    let normal_request = state.http_request(
        RequestBuilder::get("/contents.html").build(),
        &[],
        unused_callback(),
    );
    assert_eq!(normal_request.body.as_ref(), FILE_BODY);

    let alias_add_html = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
    assert_eq!(alias_add_html.body.as_ref(), FILE_BODY);

    let root_alias = state.http_request(RequestBuilder::get("/").build(), &[], unused_callback());
    assert_eq!(root_alias.body.as_ref(), INDEX_BODY);

    let empty_path_alias =
        state.http_request(RequestBuilder::get("").build(), &[], unused_callback());
    assert_eq!(empty_path_alias.body.as_ref(), INDEX_BODY);

    let subdirectory_index_alias = state.http_request(
        RequestBuilder::get("/subdirectory/index").build(),
        &[],
        unused_callback(),
    );
    assert_eq!(subdirectory_index_alias.body.as_ref(), SUBDIR_INDEX_BODY);

    let subdirectory_index_alias_2 = state.http_request(
        RequestBuilder::get("/subdirectory/").build(),
        &[],
        unused_callback(),
    );
    assert_eq!(subdirectory_index_alias_2.body.as_ref(), SUBDIR_INDEX_BODY);

    let subdirectory_index_alias_3 = state.http_request(
        RequestBuilder::get("/subdirectory").build(),
        &[],
        unused_callback(),
    );
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

    let alias_add_html = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
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

    let other_alias_still_works = state.http_request(
        RequestBuilder::get("/subdirectory/index").build(),
        &[],
        unused_callback(),
    );
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
    let alias_add_html_again = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
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

    let alias_for_other_asset_still_works = state.http_request(
        RequestBuilder::get("/subdirectory").build(),
        &[],
        unused_callback(),
    );
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

    let alias_for_other_asset_still_works = state.http_request(
        RequestBuilder::get("/subdirectory").build(),
        &[],
        unused_callback(),
    );
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

    let alias_add_html = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
    assert_eq!(alias_add_html.body.as_ref(), FILE_BODY);

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents", "text/html")
            .with_encoding("identity", vec![FILE_BODY_2])],
    );

    let alias_doesnt_overwrite_actual_file = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
    assert_eq!(
        alias_doesnt_overwrite_actual_file.body.as_ref(),
        FILE_BODY_2
    );

    state.delete_asset(DeleteAssetArguments {
        key: "/contents".to_string(),
    });

    let alias_accessible_again = state.http_request(
        RequestBuilder::get("/contents").build(),
        &[],
        unused_callback(),
    );
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

        state.create_test_asset(AssetBuilder::new("/page2.html", "text/html"));
        let response = state.fake_http_request("a-b-c.raw.icp0.io", "/page2");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.icp0.io/page2"
        );

        state.create_test_asset(AssetBuilder::new("/index.html", "text/html"));
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
    use crate::http::build_ic_certificate_expression_from_headers_and_encoding;

    use super::*;

    #[test]
    fn ic_certificate_expression_value_from_headers() {
        let h = ["a", "b", "c"].to_vec();
        let c = build_ic_certificate_expression_from_headers_and_encoding(&h, "not identity");
        assert_eq!(
            c.expression,
            r#"default_certification(ValidationArgs{certification: Certification{no_request_certification: Empty{}, response_certification: ResponseCertification{certified_response_headers: ResponseHeaderList{headers: ["content-type", "content-encoding", "a", "b", "c"]}}}})"#
        );
        let c2 = build_ic_certificate_expression_from_headers_and_encoding(&h, "identity");
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

        let v1_response = state.http_request(
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .build(),
            &[],
            unused_callback(),
        );

        assert!(
            lookup_header(&v1_response, "ic-certificateexpression").is_none(),
            "superfluous ic-certificateexpression header detected in cert v1"
        );

        let response = state.http_request(
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
            &[],
            unused_callback(),
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

        let response = state.http_request(
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
            &[],
            unused_callback(),
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
        let response = state.http_request(
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
            &[],
            unused_callback(),
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

        let response = state.http_request(
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
            &[],
            unused_callback(),
        );

        let cert_header =
            lookup_header(&response, "ic-certificate").expect("ic-certificate header missing");

        println!("IC-Certificate: {}", cert_header);

        assert!(
            cert_header.contains("version=2"),
            "cert is missing version indicator or has wrong version",
        );
        assert!(cert_header.contains("certificate=:"), "cert is missing",);
        assert!(cert_header.contains("tree=:"), "tree is missing",);
        assert!(!cert_header.contains("tree=::"), "tree is empty",);
        assert!(cert_header.contains("expr_path=:"), "expr_path is missing",);
        assert!(!cert_header.contains("expr_path=::"), "expr_path is empty",);

        assert!(cert_header == "version=2, certificate=::, tree=:2dn3gwGCBFggYqb51osZ8yEgbrtk+Z981k9J9Q0m4VEH/xmnuU6SDJqDAklodHRwX2V4cHKDAYIEWCA1sd2JIxN6F1cM5ZJxdJdNmNNEDXnePdxl5Yz/nMkXmIMCTWNvbnRlbnRzLmh0bWyDAkM8JD6DAlggwrQrUBLlYvqrQCZVjsbrUysHuLEniI92YbWT58HhfgGDAkCDAYMCWCCsJkJx/PNM4lug1TVlVDNINmk6i6Mlt5TkF2ZiU75aSoIDQIMCWCDFaHrIHl7UaWlUtBt+VDFkwpI+dahytlBeV0Be5LB6GIIDQA==:, expr_path=:2dn3g2lodHRwX2V4cHJtY29udGVudHMuaHRtbGM8JD4=:");

        create_assets(
            &mut state,
            time_now,
            vec![AssetBuilder::new("/contents.html", "text/html")
                .with_encoding("identity", vec![UPDATED_BODY])
                .with_max_age(604800)
                .with_header("Access-Control-Allow-Origin", "*")],
        );

        let response = state.http_request(
            RequestBuilder::get("/contents.html")
                .with_header("Accept-Encoding", "gzip,identity")
                .with_certificate_version(2)
                .build(),
            &[],
            unused_callback(),
        );

        let cert_header = lookup_header(&response, "ic-certificate")
            .expect("after update: ic-certificate header missing");

        println!("Updated IC-Certificate: {}", cert_header);

        assert!(cert_header == "version=2, certificate=::, tree=:2dn3gwGCBFgg1hasIZe9DV/qkwMJwOyFED/kYwg4LKtr0BWWcxuIqI6DAklodHRwX2V4cHKDAYIEWCB8ve5ZiB9SeCaYdKsv2ZfHSFZBomzvLxZtXtSxzg26iYMCTWNvbnRlbnRzLmh0bWyDAkM8JD6DAlggwrQrUBLlYvqrQCZVjsbrUysHuLEniI92YbWT58HhfgGDAkCDAYMCWCCsJkJx/PNM4lug1TVlVDNINmk6i6Mlt5TkF2ZiU75aSoIDQIMCWCC8DBBYlQxiaVAOAV6uWwZ3un2feoZJc0MW5MYdsWFsLIIDQA==:, expr_path=:2dn3g2lodHRwX2V4cHJtY29udGVudHMuaHRtbGM8JD4=:");
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

        let batch_1 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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

        let batch_id = state.create_batch(time_now);
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

        let batch_id = state.create_batch(time_now);
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

        let batch_id = state.create_batch(time_now);
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
            let batch_1 = state.create_batch(time_now);
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
                    batch_id: batch_1,
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();

            let batch_2 = state.create_batch(time_now);
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
                    batch_id: batch_2,
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();

            assert_eq!(evidence_1, evidence_2);
        }

        {
            let batch_1 = state.create_batch(time_now);
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
                    batch_id: batch_1,
                    max_iterations: Some(3),
                })
                .unwrap()
                .unwrap();

            let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_3 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_3 = state.create_batch(time_now);
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
                batch_id: batch_3,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_4 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_3 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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
                batch_id: batch_2,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_3 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(4),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
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
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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

        let batch_1 = state.create_batch(time_now);
        assert!(state
            .propose_commit_batch(CommitBatchArguments {
                batch_id: batch_1.clone(),
                operations: vec![BatchOperation::Clear(ClearArguments {}),],
            })
            .is_ok());
        let evidence_1 = state
            .compute_evidence(ComputeEvidenceArguments {
                batch_id: batch_1,
                max_iterations: Some(3),
            })
            .unwrap()
            .unwrap();

        let batch_2 = state.create_batch(time_now);
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
}
