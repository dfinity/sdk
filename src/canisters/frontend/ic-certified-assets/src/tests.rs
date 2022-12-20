use std::collections::HashMap;

use crate::http::{HttpRequest, HttpResponse, StreamingStrategy};
use crate::state_machine::{StableState, State, BATCH_EXPIRY_NANOS};
use crate::types::{
    AssetProperties, BatchId, BatchOperation, CommitBatchArguments, CreateAssetArguments,
    CreateChunkArg, DeleteAssetArguments, SetAssetContentArguments, SetAssetPropertiesArguments,
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
}

impl RequestBuilder {
    fn get(resource: impl AsRef<str>) -> Self {
        Self {
            resource: resource.as_ref().to_string(),
            method: "GET".to_string(),
            headers: vec![],
            body: ByteBuf::new(),
        }
    }

    fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers
            .push((name.as_ref().to_string(), value.as_ref().to_string()));
        self
    }

    fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.resource,
            headers: self.headers,
            body: self.body,
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
            allow_raw_access: None
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
            allow_raw_access: None
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
            allow_raw_access: None
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
            allow_raw_access: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(None),
            headers: Some(None),
            allow_raw_access: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: None,
            headers: None,
            allow_raw_access: None
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
            allow_raw_access: None
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
            allow_raw_access: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: Some(Some(HashMap::from([("new-header".into(), "value".into())]))),
            allow_raw_access: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([("new-header".into(), "value".into())])),
            allow_raw_access: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(2)),
            headers: None,
            allow_raw_access: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: Some(HashMap::from([("new-header".into(), "value".into())])),
            allow_raw_access: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: Some(None),
            allow_raw_access: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: None,
            allow_raw_access: None
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

#[ignore = "SDK-817 will enable this"]
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

    create_assets(
        &mut state,
        time_now,
        vec![AssetBuilder::new("/contents.html", "text/html")
            .with_encoding("identity", vec![FILE_BODY])
            .with_aliasing(false)],
    );

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
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/page");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.ic0.app/page"
        );

        state.create_test_asset(AssetBuilder::new("/page2.html", "text/html"));
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/page2");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.ic0.app/page2"
        );

        state.create_test_asset(AssetBuilder::new("/index.html", "text/html"));
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.ic0.app/"
        );

        let mut state = State::default();
        state.create_test_asset(
            AssetBuilder::new("/index.html", "text/html").with_allow_raw_access(Some(false)),
        );
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/");
        dbg!(&response);
        assert_eq!(response.status_code, 308);
        assert_eq!(
            lookup_header(&response, "Location").unwrap(),
            "https://a-b-c.ic0.app/"
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
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/blog.html");
        dbg!(&response);
        assert_eq!(response.status_code, 200);

        let mut state = State::default();
        state.create_test_asset(
            AssetBuilder::new("/index.html", "text/html")
                .with_encoding("identity", vec![FILE_BODY])
                .with_allow_raw_access(Some(true)),
        );
        let response = state.fake_http_request("a-b-c.raw.ic0.app", "/index.html");
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
