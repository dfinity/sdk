use crate::rc_bytes::RcBytes;
use crate::state_machine::{Asset, AssetEncoding};
use candid::{CandidType, Deserialize, Func, Nat};
use ic_certified_map::Hash;
use serde::Serialize;
use serde_bytes::ByteBuf;
use std::collections::HashMap;

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
    pub upgrade: Option<bool>,
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

#[derive(Deserialize, CandidType, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct RedirectUrl {
    pub(crate) host: Option<String>,
    pub(crate) path: Option<String>,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, PartialEq)]
pub struct HttpRedirect {
    pub from: Option<RedirectUrl>,
    pub to: RedirectUrl,
    pub response_code: u16,
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
            .find_map(|(k, v)| k.eq_ignore_ascii_case(header_key).then(|| v))
    }

    pub fn get_canister_id(&self) -> &str {
        if let Some(host_header) = self.get_header_value("Host") {
            if host_header.contains(".localhost") || host_header.contains(".app") {
                return host_header.split('.').next().unwrap();
            } else {
                if let Some(t) = self.url.split("canisterId=").nth(1) {
                    let x = t.split_once('&');
                    if let Some(c) = x {
                        return c.0;
                    }
                }
            }
        }
        "canister-id-is-unreachable"
    }

    pub fn redirect_from_raw_to_certified_domain(&self) -> HttpResponse {
        let location = format!(
            "https://{canister_id}.ic0.app{path}",
            canister_id = self.get_canister_id(),
            path = self.url
        );
        HttpResponse::build_redirect(308, location, None, false)
    }

    pub fn extract_params(&self) -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert("canisterId".to_string(), self.get_canister_id().to_string());
        if let Some(url_path) = self.url.split_once('?') {
            if !url_path.1.is_empty() {
                for (param, value) in url_path.1.split('&').filter_map(|p| p.split_once('=')) {
                    hm.insert(param.to_string(), value.to_string());
                }
            }
        }
        hm
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
            upgrade: None,
        }
    }

    pub fn build_redirect(
        status_code: u16,
        location: String,
        upgrade: Option<bool>,
        upgraded: bool,
    ) -> HttpResponse {
        if ![300, 301, 302, 303, 304, 307, 308].contains(&status_code) {
            return HttpResponse::build_400(&format!(
                "incorrect asset redirect configuration: response_code \"{}\" is not valid HTTP respone code",
                status_code
            ));
        }
        if upgrade.map_or(false, |v| v) && !upgraded {
            HttpResponse {
                status_code: 200,
                headers: vec![],
                body: RcBytes::from(ByteBuf::new()),
                streaming_strategy: None,
                upgrade,
            }
        } else {
            HttpResponse {
                status_code,
                headers: vec![("Location".to_string(), location)],
                body: RcBytes::from(ByteBuf::default()),
                streaming_strategy: None,
                upgrade,
            }
        }
    }

    pub fn build_400(err_msg: &str) -> Self {
        HttpResponse {
            status_code: 400,
            headers: vec![],
            body: RcBytes::from(ByteBuf::from(err_msg)),
            streaming_strategy: None,
            upgrade: None,
        }
    }

    pub fn build_404(certificate_header: HeaderField) -> HttpResponse {
        HttpResponse {
            status_code: 404,
            headers: vec![certificate_header],
            body: RcBytes::from(ByteBuf::from("not found")),
            streaming_strategy: None,
            upgrade: None,
        }
    }
}

impl RedirectUrl {
    pub fn get_location_url(&self, query_params: HashMap<String, String>) -> String {
        let mut location_url = String::new();
        if let Some(host) = &self.host {
            if host.starts_with("http") {
                location_url.push_str(host);
            } else {
                location_url.push_str(&format!("https://{}", host))
            };
        }
        if let Some(path) = &self.path {
            location_url.push_str(path);
        }
        for (query_param_key, query_param_value) in query_params {
            let replace_key =  format!("{{{}}}", &query_param_key);
            location_url = location_url.replace(&replace_key, &query_param_value)
        }
        location_url
    }
}

impl HttpRedirect {
    pub fn redirect(
        redirect_config: &Option<Self>,
        allow_raw_access: Option<bool>,
        req: &HttpRequest,
        upgraded: bool,
    ) -> Option<HttpResponse> {
        let redirect_from_raw = allow_raw_access.map_or(false, |v| v)
            && req
                .get_header_value("Host")
                .map_or(false, |v| v.contains(".raw.ic"));

        if !redirect_from_raw && redirect_config.is_none() {
            return None;
        } else if redirect_from_raw && redirect_config.is_none() {
            return Some(req.redirect_from_raw_to_certified_domain());
        }

        let redirect_config = redirect_config.clone().unwrap().clone();

        if let Err(e) = redirect_config.is_valid() {
            return Some(HttpResponse::build_400(&e));
        }

        if let Some(mut location) = redirect_config.determine_redirect_url(req) {
            let req_host = req.get_header_value("Host");
            let query_params = req.extract_params();
            if req_host.is_none() {
                return Some(HttpResponse::build_400("Host header is missing"));
            }
            let upgrade = req_host.and_then(|host| {
                let is_non_local_deployment = host.contains("ic0.app")
                    || host.contains("ic1.app")
                    || host.contains("ic0.local")
                    || host.contains("ic1.local");
                let is_certified = !host.contains(".raw.ic");
                Some(is_non_local_deployment && is_certified)
            });
            if let Some(to_host) = location.host.as_ref() {
                if redirect_from_raw {
                    location.host = Some(to_host.replace(".raw.ic", ".ic"));
                }
            }
            let location_url = location.get_location_url(query_params);
            if Self::check_cyclic_redirection(&location_url, req, allow_raw_access).is_err()
            {
                return None;
            }
            Some(HttpResponse::build_redirect(
                redirect_config.response_code,
                location_url,
                upgrade,
                upgraded,
            ))
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> Result<(), String> {
        if self.from.is_some() {
            if self.from.as_ref().unwrap().host.is_none()
                && self.from.as_ref().unwrap().path.is_none()
            {
                return Err("AssetRedirect.from must have a host or path".to_string());
            }
        }
        if self.to.host.is_none() && self.to.path.is_none() {
            return Err("AssetRedirect.to must have a host or path".to_string());
        }
        Ok(())
    }

    pub fn determine_redirect_url(&self, req: &HttpRequest) -> Option<RedirectUrl> {
        macro_rules! get_redirect_url {
            ($to:expr, $req:expr) => {
                let mut redirect_url = RedirectUrl {
                    host: None,
                    path: None
                };
                if let Some(host) = &$to.host {
                    if host.contains("http") {
                        redirect_url.host = Some(host.clone());
                    } else {
                        redirect_url.host = Some(format!("https://{}", host));
                    }
                }
                if let Some(path) = &$to.path {
                    redirect_url.path = Some(path.clone());
                } else {
                    redirect_url.path = Some($req.url.clone());
                }
                return Some(redirect_url);
            };
        }

        macro_rules! from_host_is_some_and_contains_request_host {
            ($from_host:expr, $req:expr) => {
                $from_host.is_some()
                    && $req
                        .get_header_value("Host")
                        .map_or(false, |v| v.contains(&$from_host.clone().unwrap()))
            };
        }

        macro_rules! from_path_is_some_and_contains_request_path {
            ($from_path:expr, $req:expr) => {
                $from_path.is_some() && $req.url.contains(&$from_path.clone().unwrap())
            };
        }

        match &self.from {
            Some(RedirectUrl {
                host: from_host,
                path: from_path,
            }) if from_host_is_some_and_contains_request_host!(from_host, req)
                && from_path_is_some_and_contains_request_path!(from_path, req) =>
            {
                get_redirect_url!(self.to, req);
            }
            Some(RedirectUrl {
                host: from_host,
                path: from_path,
            }) if from_host_is_some_and_contains_request_host!(from_host, req)
                && from_path.is_none() =>
            {
                get_redirect_url!(self.to, req);
            }
            Some(RedirectUrl {
                host: from_host,
                path: from_path,
            }) if from_path_is_some_and_contains_request_path!(from_path, req)
                && from_host.is_none() =>
            {
                get_redirect_url!(self.to, req);
            }
            None => {
                get_redirect_url!(self.to, req);
            }
            Some(_) => None,
        }
    }

    pub fn check_cyclic_redirection(
        location_url: &str,
        request: &HttpRequest,
        allow_raw_access: Option<bool>,
    ) -> Result<(), String> {
        if location_url.starts_with('/') && (location_url == request.get_path() || location_url == request.url) {
                return Err(format!(
                    "redirect loop: {:?} -> {}",
                    &location_url,
                    request.url
                ));

        } else {
            let request_url = format!(
                "{}{}",
                request.get_header_value("Host").unwrap_or(&"".to_string()),
                request.url
            );
            let request_url = request_url
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/');
            let location_url = location_url
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/');
            if request_url == location_url {
                return Err(format!(
                    "redirect loop: {:?} -> {}",
                    &location_url, request_url
                ));
            } else if allow_raw_access.map_or(false, |v| v)
                && location_url == request_url.replace(".raw.ic", ".ic")
            {
                return Err(
                "redirect loop: the request will be continously redirecting from raw to certified domain, then from certified to raw domain".to_string(),
            );
            }
        }
        Ok(())
    }
}
