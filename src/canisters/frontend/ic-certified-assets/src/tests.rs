#![allow(dead_code)]
use std::collections::HashMap;

use crate::http::{HttpRedirect, HttpRequest, HttpResponse, StreamingStrategy};
use crate::state_machine::{StableState, State, BATCH_EXPIRY_NANOS};
use crate::types::{
    BatchId, BatchOperation, CommitBatchArguments, CreateAssetArguments, CreateChunkArg,
    DeleteAssetArguments, SetAssetContentArguments,
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
    redirect: Option<HttpRedirect>,
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
            redirect: None,
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

    fn with_redirect(mut self, redirect: HttpRedirect) -> Self {
        self.redirect = Some(redirect);
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
            redirect: asset.redirect,
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
        .find_map(|(h, v)| h.eq_ignore_ascii_case(header).then(|| v.as_str()))
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
            )]))
        })
    );
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(604800),
            headers: Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )]))
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(1)),
            headers: Some(Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )])))
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )]))
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(None),
            headers: Some(None)
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: None,
            headers: None
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(1)),
            headers: Some(Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )])))
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([(
                "X-Content-Type-Options".into(),
                "nosniff".into()
            )]))
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: Some(Some(HashMap::from([("new-header".into(), "value".into())])))
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(1),
            headers: Some(HashMap::from([("new-header".into(), "value".into())]))
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: Some(Some(2)),
            headers: None
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: Some(HashMap::from([("new-header".into(), "value".into())]))
        })
    );

    assert!(state
        .set_asset_properties(SetAssetPropertiesArguments {
            key: "/max-age.html".into(),
            max_age: None,
            headers: Some(None)
        })
        .is_ok());
    assert_eq!(
        state.get_asset_properties("/max-age.html".into()),
        Ok(AssetProperties {
            max_age: Some(2),
            headers: None
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
mod test_http_redirects {
    use super::{create_assets, unused_callback, AssetBuilder, RequestBuilder};
    use crate::{
        http::{HttpRedirect, HttpResponse, RedirectUrl},
        state_machine::State,
    };

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    impl State {
        fn fake_http_request(&self, host: &str, path: &str) -> (HttpResponse, bool) {
            let fake_cert = [0xca, 0xfe];
            let mut upgraded = false;
            let mut resp = self.http_request(
                RequestBuilder::get(path).with_header("Host", host).build(),
                &fake_cert,
                unused_callback(),
            );
            // emulate service worker behaviour
            if resp.upgrade.map_or(false, |v| v) {
                upgraded = true;
                resp = self.http_request_update(
                    RequestBuilder::get(path).with_header("Host", host).build(),
                );
            }
            (resp, upgraded)
        }

        fn create_test_asset(&mut self, asset: AssetBuilder) {
            create_assets(self, 100_000_000_000, vec![asset]);
        }
    }

    #[test]
    fn correct_redirect_codes() {
        for response_code in vec![300, 301, 302, 303, 304, 307, 308] {
            let mut state = State::default();
            state.create_test_asset(
                AssetBuilder::new("/redirect.html", "text/html").with_redirect(HttpRedirect {
                    to: RedirectUrl::new(
                        Some("www.example.com".to_string()),
                        Some("/redirected.html".to_string()),
                    ),
                    response_code,
                    ..Default::default()
                }),
            );
            let (response, _) = state.fake_http_request("www.example.com", "/redirect.html");
            assert_eq!(response.status_code, response_code);
        }
    }

    #[test]
    fn incorrect_redirect_codes() {
        for response_code in vec![200, 305, 306, 309, 310, 311, 400, 404, 405, 500] {
            let mut state = State::default();
            state.create_test_asset(
                AssetBuilder::new("/redirect.html", "text/html").with_redirect(HttpRedirect {
                    to: RedirectUrl::new(
                        Some("www.example.com".to_string()),
                        Some("/redirected.html".to_string()),
                    ),
                    response_code,
                    ..Default::default()
                }),
            );
            let (response, _) = state.fake_http_request("www.example.com", "/redirect.html");
            assert_eq!(response.status_code, 400);
        }
    }

    #[test]
    fn redirect_struct_validity_checks() {
        let a = HttpRedirect {
            from: Some(RedirectUrl::new(None, None)),
            to: RedirectUrl::new(None, None),
            response_code: 11111,
        };
        assert!(a.is_valid().is_err());

        let a = HttpRedirect {
            to: RedirectUrl::new(Some("".to_string()), None),
            ..Default::default()
        };
        assert!(a.is_valid().is_ok());

        let a = HttpRedirect {
            to: RedirectUrl::new(Some("x".to_string()), None),
            ..Default::default()
        };
        assert!(a.is_valid().is_ok());

        let a = HttpRedirect {
            to: RedirectUrl::new(Some("x".to_string()), None),
            ..Default::default()
        };
        assert!(a.is_valid().is_ok());
    }

    #[test]
    fn redirects() {
        type RoutingTestCase<'a> = (
            Option<&'a str>,
            Option<&'a str>,
            Option<&'a str>,
            Option<&'a str>,
            &'a str,
            &'a str,
            Option<bool>,
            Option<&'a str>,
            bool,
        );

        impl HttpRedirect {
            fn from_test_case(t: RoutingTestCase) -> Self {
                Self {
                    from: if t.0.is_none() && t.1.is_none() {
                        None
                    } else {
                        Some(RedirectUrl {
                            host: t.0.map(|v| v.into()),
                            path: t.1.map(|v| v.into()),
                        })
                    },
                    to: RedirectUrl {
                        host: t.2.map(|v| v.into()),
                        path: t.3.map(|v| v.into()),
                    },
                    response_code: 308,
                }
            }
        }

        #[rustfmt::skip]
        let basic_test_routing_table: Vec<RoutingTestCase> = vec![
            // Inputs ----------------------------------------------------------------------------------------------------------------- | Outputs ------------------------------------------------------
/* index */ // from_host  | from_path   | to_host                     | to_path      | request_host   | request_path | allow_raw_access | Location header value              | via http_request_upgrade?
/* 0 */     (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/home"),  true),
/* 1 */     (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/blog"),  true),
/* 2 */     (Some("a-b-c"), Some("blog"), None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("/home"),                       true),
/* 3 */     (Some("a-b-c"), Some("blog"), None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 4 */     (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/home"),  true),
/* 5 */     (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/blog"),  true),
/* 6 */     (Some("a-b-c"), None        , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("/home"),                       true),
/* 7 */     (Some("a-b-c"), None        , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 8 */     (None         , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/home"),  true),
/* 9 */     (None         , Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/blog"),  true),
/* 10 */    (None         , Some("blog"), None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("/home"),                       true),
/* 11 */    (None         , Some("blog"), None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 12 */    (None         , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/home"),  true),
/* 13 */    (None         , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , Some("https://a-b-c.xyz.app/blog"),  true),
/* 14 */    (None         , None        , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , Some("/home"),                       true),
/* 15 */    (None         , None        , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 16 */    (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 17 */    (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 18 */    (Some("inv")  , Some("blog"), None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 19 */    (Some("inv")  , Some("blog"), None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 20 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 21 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 22 */    (Some("inv")  , None        , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 23 */    (Some("inv")  , None        , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 24 */    (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 25 */    (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 26 */    (Some("a-b-c"), Some("inv") , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 27 */    (Some("a-b-c"), Some("inv") , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 28 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 29 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 30 */    (None         , Some("inv") , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 31 */    (None         , Some("inv") , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 32 */    (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 33 */    (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 34 */    (Some("inv")  , Some("inv") , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 35 */    (Some("inv")  , Some("inv") , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 36 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 37 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 38 */    (Some("inv")  , None        , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 39 */    (Some("inv")  , None        , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 40 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 41 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 42 */    (None         , Some("inv") , None                        , Some("/home"), "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
/* 43 */    (None         , Some("inv") , None                        , None         , "a-b-c.ic0.app", "/blog",       Some(true)       , None,                                false),
        ];

        #[rustfmt::skip]
        let forbid_raw_access_test_routing_table: Vec<RoutingTestCase> = vec![
            // Inputs --------------------------------------------------------------------------------------------------------------------- | Outputs ------------------------------------------------------
/* index */ // from_host  | from_path   | to_host                     | to_path      | request_host       | request_path | allow_raw_access | Location header value              | via http_request_upgrade?
/* 44 */    (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/home"),  false),
/* 45 */    (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/blog"),  false),
/* 46 */    (Some("a-b-c"), Some("blog"), None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("/home"),                       false),
/* 47 */    (Some("a-b-c"), Some("blog"), None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 48 */    (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/home"),  false),
/* 49 */    (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/blog"),  false),
/* 50 */    (Some("a-b-c"), None        , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("/home"),                       false),
/* 51 */    (Some("a-b-c"), None        , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 52 */    (None         , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/home"),  false),
/* 53 */    (None         , Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/blog"),  false),
/* 54 */    (None         , Some("blog"), None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("/home"),                       false),
/* 55 */    (None         , Some("blog"), None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 56 */    (None         , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/home"),  false),
/* 57 */    (None         , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("https://a-b-c.xyz.app/blog"),  false),
/* 58 */    (None         , None        , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , Some("/home"),                       false),
/* 59 */    (None         , None        , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 60 */    (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 61 */    (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 62 */    (Some("inv")  , Some("blog"), None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 63 */    (Some("inv")  , Some("blog"), None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 64 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 65 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 66 */    (Some("inv")  , None        , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 67 */    (Some("inv")  , None        , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 68 */    (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 69 */    (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 70 */    (Some("a-b-c"), Some("inv") , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 71 */    (Some("a-b-c"), Some("inv") , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 72 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 73 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 74 */    (None         , Some("inv") , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 75 */    (None         , Some("inv") , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 76 */    (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 77 */    (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 78 */    (Some("inv")  , Some("inv") , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 79 */    (Some("inv")  , Some("inv") , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 80 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 81 */    (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 82 */    (Some("inv")  , None        , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 83 */    (Some("inv")  , None        , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 84 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 85 */    (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 86 */    (None         , Some("inv") , None                        , Some("/home"), "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
/* 87 */    (None         , Some("inv") , None                        , None         , "a-b-c.raw.ic0.app", "/blog",       Some(false)      , None,                                false),
        ];

        #[rustfmt::skip]
        let localhost_test_routing_table: Vec<RoutingTestCase> = vec![
            // Inputs ----------------------------------------------------------------------------------------------------------------------------------- | Outputs ----------------------------------------------------------
/* index */ // from_host  | from_path   | to_host                     | to_path      | request_host          | request_path            | allow_raw_access | Location header value                              | via http_request_upgrade?
/* 88 */    (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4349", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/home")                 ,  false),
/* 89 */    (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/blog")                 ,  false),
/* 90 */    (Some("a-b-c"), Some("blog"), None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("/home")                                      ,  false),
/* 91 */    (Some("a-b-c"), Some("blog"), None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 92 */    (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/home")                 ,  false),
/* 93 */    (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/blog")                 ,  false),
/* 94 */    (Some("a-b-c"), None        , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("/home")                                      ,  false),
/* 95 */    (Some("a-b-c"), None        , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 96 */    (None         , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/home")                 ,  false),
/* 97 */    (None         , Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/blog")                 ,  false),
/* 98 */    (None         , Some("blog"), None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("/home")                                      ,  false),
/* 99 */    (None         , Some("blog"), None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 100 */   (None         , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/home")                 ,  false),
/* 101 */   (None         , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("https://a-b-c.xyz.app/blog")                 ,  false),
/* 102 */   (None         , None        , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , Some("/home")                                      ,  false),
/* 103 */   (None         , None        , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 104 */   (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 105 */   (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 106 */   (Some("inv")  , Some("blog"), None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 107 */   (Some("inv")  , Some("blog"), None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 108 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 109 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 110 */   (Some("inv")  , None        , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 111 */   (Some("inv")  , None        , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 112 */   (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 113 */   (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 114 */   (Some("a-b-c"), Some("inv") , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 115 */   (Some("a-b-c"), Some("inv") , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 116 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 117 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 118 */   (None         , Some("inv") , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 119 */   (None         , Some("inv") , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 120 */   (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 121 */   (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 122 */   (Some("inv")  , Some("inv") , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 123 */   (Some("inv")  , Some("inv") , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 124 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 125 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 126 */   (Some("inv")  , None        , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 127 */   (Some("inv")  , None        , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 128 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 129 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 130 */   (None         , Some("inv") , None                        , Some("/home"), "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 131 */   (None         , Some("inv") , None                        , None         , "a-b-c.localhost:4348", "/blog"                 , Some(false)      , None                                               ,  false),
/* 132 */   (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 133 */   (Some("a-b-c"), Some("blog"), Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 134 */   (Some("a-b-c"), Some("blog"), None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 135 */   (Some("a-b-c"), Some("blog"), None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 136 */   (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 137 */   (Some("a-b-c"), None        , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 138 */   (Some("a-b-c"), None        , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 139 */   (Some("a-b-c"), None        , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 140 */   (None         , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , Some("https://a-b-c.xyz.app/home")                 ,  false),
/* 141 */   (None         , Some("blog"), Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , Some("https://a-b-c.xyz.app/blog?canisterId=a-b-c"),  false),
/* 142 */   (None         , Some("blog"), None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , Some("/home")                                      ,  false),
/* 143 */   (None         , Some("blog"), None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 144 */   (None         , None        , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , Some("https://a-b-c.xyz.app/home")                 ,  false),
/* 145 */   (None         , None        , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , Some("https://a-b-c.xyz.app/blog?canisterId=a-b-c"),  false),
/* 146 */   (None         , None        , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , Some("/home")                                      ,  false),
/* 147 */   (None         , None        , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 148 */   (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 149 */   (Some("inv")  , Some("blog"), Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 150 */   (Some("inv")  , Some("blog"), None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 151 */   (Some("inv")  , Some("blog"), None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 152 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 153 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 154 */   (Some("inv")  , None        , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 155 */   (Some("inv")  , None        , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 156 */   (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 157 */   (Some("a-b-c"), Some("inv") , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 158 */   (Some("a-b-c"), Some("inv") , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 159 */   (Some("a-b-c"), Some("inv") , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 160 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 161 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 162 */   (None         , Some("inv") , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 163 */   (None         , Some("inv") , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 164 */   (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 165 */   (Some("inv")  , Some("inv") , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 166 */   (Some("inv")  , Some("inv") , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 167 */   (Some("inv")  , Some("inv") , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 168 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 169 */   (Some("inv")  , None        , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 170 */   (Some("inv")  , None        , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 171 */   (Some("inv")  , None        , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 172 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 173 */   (None         , Some("inv") , Some("{canisterId}.xyz.app"), None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 174 */   (None         , Some("inv") , None                        , Some("/home"), "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
/* 175 */   (None         , Some("inv") , None                        , None         , "localhost:4348"      , "/blog?canisterId=a-b-c", Some(false)      , None                                               ,  false),
        ];

        #[rustfmt::skip]
        let cyclic_redirects_test_routing_table: Vec<RoutingTestCase> = vec![
            // Inputs ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | Outputs ----------------------------------------
/* index */ // from_host      | from_path   | to_host                            | to_path                              | request_host          | request_path            | allow_raw_access | Location header value | via http_request_upgrade?
/* 176 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.ic0.app")       , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 177 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.ic0.app")       , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 178 */   (Some("a-b-c")    , Some("blog"), None                               , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 179 */   (Some("a-b-c")    , Some("blog"), None                               , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 180 */   (Some("a-b-c")    , None        , Some("{canisterId}.ic0.app")       , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 181 */   (Some("a-b-c")    , None        , Some("{canisterId}.ic0.app")       , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 182 */   (Some("a-b-c")    , None        , None                               , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 183 */   (Some("a-b-c")    , None        , None                               , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 184 */   (None             , Some("blog"), Some("{canisterId}.ic0.app")       , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 185 */   (None             , Some("blog"), Some("{canisterId}.ic0.app")       , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 186 */   (None             , Some("blog"), None                               , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 187 */   (None             , Some("blog"), None                               , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 188 */   (None             , None        , Some("{canisterId}.ic0.app")       , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 189 */   (None             , None        , Some("{canisterId}.ic0.app")       , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 190 */   (None             , None        , None                               , Some("/blog")                        , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 191 */   (None             , None        , None                               , None                                 , "a-b-c.ic0.app"       , "/blog"                 , Some(false)      , None                  , false),
/* 192 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 193 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 194 */   (Some("a-b-c")    , Some("blog"), None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 195 */   (Some("a-b-c")    , Some("blog"), None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 196 */   (Some("a-b-c")    , None        , Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 197 */   (Some("a-b-c")    , None        , Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 198 */   (Some("a-b-c")    , None        , None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 199 */   (Some("a-b-c")    , None        , None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 200 */   (None             , Some("blog"), Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 201 */   (None             , Some("blog"), Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 202 */   (None             , Some("blog"), None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 203 */   (None             , Some("blog"), None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 204 */   (None             , None        , Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 205 */   (None             , None        , Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 206 */   (None             , None        , None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 207 */   (None             , None        , None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(false)      , None                  , false),
/* 208 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 209 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 210 */   (Some("a-b-c")    , Some("blog"), None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 211 */   (Some("a-b-c")    , Some("blog"), None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 212 */   (Some("a-b-c")    , None        , Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 213 */   (Some("a-b-c")    , None        , Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 214 */   (Some("a-b-c")    , None        , None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 215 */   (Some("a-b-c")    , None        , None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 216 */   (None             , Some("blog"), Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 217 */   (None             , Some("blog"), Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 218 */   (None             , Some("blog"), None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 219 */   (None             , Some("blog"), None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 220 */   (None             , None        , Some("{canisterId}.raw.ic0.app")   , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 221 */   (None             , None        , Some("{canisterId}.raw.ic0.app")   , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 222 */   (None             , None        , None                               , Some("/blog")                        , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 223 */   (None             , None        , None                               , None                                 , "a-b-c.raw.ic0.app"   , "/blog"                 , Some(true)       , None                  , false),
/* 224 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.localhost:4349"), Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 225 */   (Some("a-b-c")    , Some("blog"), Some("{canisterId}.localhost:4349"), None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 226 */   (Some("a-b-c")    , Some("blog"), None                               , Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 227 */   (Some("a-b-c")    , Some("blog"), None                               , None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 228 */   (Some("a-b-c")    , None        , Some("{canisterId}.localhost:4349"), Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 229 */   (Some("a-b-c")    , None        , Some("{canisterId}.localhost:4349"), None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 230 */   (Some("a-b-c")    , None        , None                               , Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 231 */   (Some("a-b-c")    , None        , None                               , None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 232 */   (None             , Some("blog"), Some("{canisterId}.localhost:4349"), Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 233 */   (None             , Some("blog"), Some("{canisterId}.localhost:4349"), None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 234 */   (None             , Some("blog"), None                               , Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 235 */   (None             , Some("blog"), None                               , None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 236 */   (None             , None        , Some("{canisterId}.localhost:4349"), Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 237 */   (None             , None        , Some("{canisterId}.localhost:4349"), None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 238 */   (None             , None        , None                               , Some("/blog")                        , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 239 */   (None             , None        , None                               , None                                 , "a-b-c.localhost:4349", "/blog"                 , Some(false)      , None                  , false),
/* 240 */   (Some("localhost"), Some("blog"), Some("localhost:4349")             , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 241 */   (Some("localhost"), Some("blog"), Some("localhost:4349")             , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 242 */   (Some("localhost"), Some("blog"), None                               , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 243 */   (Some("localhost"), Some("blog"), None                               , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 244 */   (Some("localhost"), None        , Some("localhost:4349")             , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 245 */   (Some("localhost"), None        , Some("localhost:4349")             , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 246 */   (Some("localhost"), None        , None                               , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 247 */   (Some("localhost"), None        , None                               , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 248 */   (None             , Some("blog"), Some("localhost:4349")             , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 249 */   (None             , Some("blog"), Some("localhost:4349")             , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 250 */   (None             , Some("blog"), None                               , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 251 */   (None             , Some("blog"), None                               , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 252 */   (None             , None        , Some("localhost:4349")             , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 253 */   (None             , None        , Some("localhost:4349")             , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 254 */   (None             , None        , None                               , Some("/blog?canisterId={canisterId}"), "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
/* 255 */   (None             , None        , None                               , None                                 , "localhost:4349"      , "/blog?canisterId=a-b-c", Some(false)      , None                  , false),
        ];

        #[rustfmt::skip]
        let query_params_test_routing_table: Vec<RoutingTestCase> = vec![
            // Inputs ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | Outputs ---------------------------------------------------------------------------------------------------------
/* index */ // from_host  | from_path   | to_host                                | to_path                                | request_host    | request_path                                        | allow_raw_access | Location header value                                                                 | via http_request_upgrade?
/* 256 */   (None         , None        , None                                   , Some("/{param1}?{param2}={canisterId}"), "localhost:4349", "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("/value1?value2=a-b-c")                                                          , false),
/* 257 */   (None         , None        , Some("/{param1}?{param2}={canisterId}"), Some("/{param1}?{param2}={canisterId}"), "localhost:4349", "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/value1?value2=a-b-c")                              , false),
/* 258 */   (None         , None        , Some("/{param1}?{param2}={canisterId}"), None                                   , "localhost:4349", "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/blog?canisterId=a-b-c&param1=value1&param2=value2"), false),
/* 259 */   (None         , None        , None                                   , Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("/value1?value2=a-b-c")                                                          , true),
/* 260 */   (None         , None        , Some("/{param1}?{param2}={canisterId}"), Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/value1?value2=a-b-c")                              , true),
/* 261 */   (None         , None        , Some("/{param1}?{param2}={canisterId}"), None                                   , "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/blog?canisterId=a-b-c&param1=value1&param2=value2"), true),
/* 262 */   (Some("a-b-c"), None        , None                                   , Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("/value1?value2=a-b-c")                                                          , true),
/* 263 */   (Some("a-b-c"), None        , Some("/{param1}?{param2}={canisterId}"), Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/value1?value2=a-b-c")                              , true),
/* 264 */   (Some("a-b-c"), None        , Some("/{param1}?{param2}={canisterId}"), None                                   , "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/blog?canisterId=a-b-c&param1=value1&param2=value2"), true),
/* 265 */   (None         , Some("blog"), None                                   , Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("/value1?value2=a-b-c")                                                          , true),
/* 266 */   (None         , Some("blog"), Some("/{param1}?{param2}={canisterId}"), Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/value1?value2=a-b-c")                              , true),
/* 267 */   (None         , Some("blog"), Some("/{param1}?{param2}={canisterId}"), None                                   , "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/blog?canisterId=a-b-c&param1=value1&param2=value2"), true),
/* 268 */   (Some("a-b-c"), Some("blog"), None                                   , Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("/value1?value2=a-b-c")                                                          , true),
/* 269 */   (Some("a-b-c"), Some("blog"), Some("/{param1}?{param2}={canisterId}"), Some("/{param1}?{param2}={canisterId}"), "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/value1?value2=a-b-c")                              , true),
/* 270 */   (Some("a-b-c"), Some("blog"), Some("/{param1}?{param2}={canisterId}"), None                                   , "a-b-c.ic0.app" , "/blog?canisterId=a-b-c&param1=value1&param2=value2", Some(false)      , Some("https:///value1?value2=a-b-c/blog?canisterId=a-b-c&param1=value1&param2=value2"), true),
        ];

        let time_now = 100_000_000_000;

        for (idx, test_case) in [
            &basic_test_routing_table[..],
            &forbid_raw_access_test_routing_table[..],
            &localhost_test_routing_table[..],
            &cyclic_redirects_test_routing_table[..],
            &query_params_test_routing_table[..],
        ]
        .concat()
        .iter()
        .enumerate()
        {
            let asset_path = match test_case.5.find('?') {
                Some(i) => &test_case.5[..i],
                None => &test_case.5[..],
            };
            let mut state = State::default();
            create_assets(
                &mut state,
                time_now,
                vec![AssetBuilder::new(asset_path, "text/html")
                    .with_encoding("identity", vec![BODY])
                    .with_allow_raw_access(test_case.6)
                    .with_redirect(HttpRedirect::from_test_case(*test_case))],
            );
            let (resp, was_upgraded) = state.fake_http_request(test_case.4, test_case.5);
            let location_header = resp
                .headers
                .iter()
                .find(|x| x.0.contains("Location"))
                .map(|v| v.1.as_str());
            assert_eq!(
                location_header, test_case.7,
                "test case #{} doesn't redirect correctly",
                idx
            );
            assert_eq!(
                was_upgraded,
                test_case.8,
                "test case #{idx} was{verb_negation} supposed to redirect via http_request_update method",
                idx = idx,
                verb_negation = if test_case.8 { "" } else { "n't" }
            );
        }
    }
}
