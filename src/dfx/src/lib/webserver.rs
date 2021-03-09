use crate::error_unknown;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::locations::canister_did_location;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::check_candid_file;

mod http_transport;

use actix::System;
use actix_cors::Cors;
use actix_server::Server;
use actix_web::client::{Client, ClientBuilder, Connector};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::{StatusCode, Uri};
use actix_web::{
    http, middleware, web, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::anyhow;
use crossbeam::channel::Sender;
use futures::StreamExt;
use ic_agent::Agent;
use ic_types::Principal;
use ic_utils::call::SyncCall;
use ic_utils::interfaces::http_request::HeaderField;
use serde::Deserialize;
use slog::{debug, info, trace, Logger};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use url::Url;

/// The amount of time to wait for the client to answer, in seconds.
/// Actix requests does not support having no timeout, so we have to put a reasonable value here,
/// even though our normal canister commands don't have timeouts themselves.
const FORWARD_REQUEST_TIMEOUT_IN_SECS: u64 = 60;

struct ForwardActixData {
    pub providers: Vec<Url>,
    pub logger: slog::Logger,
    pub counter: usize,
}

struct CandidData {
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
}

struct HttpRequestData {
    pub bind: SocketAddr,
    pub logger: slog::Logger,
}

#[derive(Deserialize)]
enum Format {
    #[serde(rename = "js")]
    Javascript,
    #[serde(rename = "ts")]
    Typescript,
}

#[derive(Deserialize)]
struct CandidRequest {
    #[serde(rename = "canisterId")]
    canister_id: String,
    format: Option<Format>,
}

async fn candid(
    web::Query(info): web::Query<CandidRequest>,
    data: web::Data<Arc<CandidData>>,
) -> Result<HttpResponse, Error> {
    let id = info.canister_id;
    let network_descriptor = &data.network_descriptor;
    let store =
        CanisterIdStore::for_network(&network_descriptor).map_err(ErrorInternalServerError)?;

    let candid_path = store
        .get_name(&id)
        .map(|canister_name| canister_did_location(&data.build_output_root, &canister_name))
        .ok_or_else(|| {
            anyhow!(
                "Cannot find canister {} for network {}",
                id,
                network_descriptor.name.clone()
            )
        })
        .map_err(ErrorInternalServerError)?
        .canonicalize()
        .map_err(|_e| anyhow!("Cannot find candid file."))
        .map_err(ErrorInternalServerError)?;

    let content = match info.format {
        None => std::fs::read_to_string(candid_path).map_err(ErrorInternalServerError)?,
        Some(Format::Javascript) => {
            let (env, ty) = check_candid_file(&candid_path).map_err(ErrorInternalServerError)?;
            candid::bindings::javascript::compile(&env, &ty)
        }
        Some(Format::Typescript) => {
            let (env, ty) = check_candid_file(&candid_path).map_err(ErrorInternalServerError)?;
            candid::bindings::typescript::compile(&env, &ty)
        }
    };
    let response = HttpResponse::Ok().body(content);
    Ok(response)
}

async fn forward(
    req: HttpRequest,
    mut payload: web::Payload,
    client: web::Data<Client>,
    actix_data: web::Data<Arc<Mutex<ForwardActixData>>>,
) -> Result<HttpResponse, Error> {
    let mut data = actix_data.lock().unwrap();
    data.counter += 1;
    let count = data.counter;

    let mut url = data.providers[count % data.providers.len()].clone();
    url.set_path(req.uri().path());
    url.set_query(req.uri().query());

    let forwarded_req = client
        .request_from(url.as_str(), req.head())
        .no_decompress()
        .timeout(std::time::Duration::from_secs(
            FORWARD_REQUEST_TIMEOUT_IN_SECS,
        ));

    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    // Set the virtual host properly.
    let forwarded_req = if let Some(h) = url.host() {
        forwarded_req.header("host", h.to_string())
    } else {
        forwarded_req
    };

    let logger = data.logger.clone();

    let mut req_body = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        req_body.extend_from_slice(&item?);
    }
    debug!(
        logger,
        "Request ({}) to replica ({})",
        indicatif::HumanBytes(req_body.len() as u64),
        url,
    );
    trace!(logger, "  headers");
    for (k, v) in req.head().headers.iter() {
        trace!(logger, "      {}: {}", k, v.to_str().unwrap());
    }
    trace!(logger, "  body    {}", hex::encode(&req_body));
    let mut response = forwarded_req
        .send_body(req_body)
        .await
        .map_err(Error::from)?;

    let mut payload = response.take_payload();
    let mut resp_body = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        resp_body.extend_from_slice(&item?);
    }

    let mut client_resp = HttpResponse::build(response.status());
    for (header_name, header_value) in response
        .headers()
        .iter()
        .filter(|(h, _)| *h != "connection" && *h != "content-length")
    {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    debug!(
        logger,
        "Response ({}) with status code {}",
        indicatif::HumanBytes(resp_body.len() as u64),
        response.status().as_u16()
    );
    trace!(logger, "  type  {}", response.content_type());
    trace!(logger, "  body  {}", hex::encode(&resp_body));

    Ok(client_resp.body(resp_body))
}

fn resolve_canister_id_from_hostname(hostname: &str) -> Option<Principal> {
    let url = Uri::from_str(hostname).ok()?;

    // Check if it's localhost or ic0.
    match url.host()?.split('.').collect::<Vec<&str>>().as_slice() {
        [.., maybe_canister_id, "localhost"] | [.., maybe_canister_id, "ic0", "app"] => {
            if let Ok(canister_id) = Principal::from_text(maybe_canister_id) {
                return Some(canister_id);
            }
        }
        _ => {}
    };

    None
}

fn resolve_canister_id_from_query(url: &Uri) -> Option<Principal> {
    let (_, canister_id) = url::form_urlencoded::parse(url.query()?.as_bytes())
        .find(|(name, _)| name == "canisterId")?;
    Principal::from_text(canister_id.as_ref()).ok()
}

fn resolve_canister_id(request: &HttpRequest) -> Option<Principal> {
    // Look for subdomains if there's a host header.
    if let Some(host_header) = request.headers().get("Host") {
        if let Ok(host) = host_header.to_str() {
            if let Some(canister_id) = resolve_canister_id_from_hostname(host) {
                return Some(canister_id);
            }
        }
    }

    // Look into the URI.
    if let Some(canister_id) = resolve_canister_id_from_query(request.uri()) {
        return Some(canister_id);
    }

    // Look into the request by header.
    if let Some(referer_header) = request.headers().get("referer") {
        if let Ok(referer) = referer_header.to_str() {
            if let Ok(referer_uri) = Uri::from_str(referer) {
                if let Some(canister_id) = resolve_canister_id_from_query(&referer_uri) {
                    return Some(canister_id);
                }
            }
        }
    }

    None
}

/// HTTP Request route. See
/// https://www.notion.so/Design-HTTP-Canisters-Queries-d6bc980830a947a88bf9148a25169613
async fn http_request(
    req: HttpRequest,
    mut payload: web::Payload,
    http_request_data: web::Data<Arc<HttpRequestData>>,
) -> Result<HttpResponse, Error> {
    let logger = http_request_data.logger.clone();
    let transport = http_transport::ActixWebClientHttpTransport::create(format!(
        "http://{}",
        http_request_data.bind.ip().to_string()
    ))
    .map_err(|err| actix_web::error::InternalError::new(err, StatusCode::INTERNAL_SERVER_ERROR))?;

    // We need to convert errors into 500s, which are regular Ok(Response).
    let agent = match Agent::builder().with_transport(transport).build() {
        Ok(agent) => agent,
        Err(err) => {
            return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Details: {:?}", err)));
        }
    };

    let canister_id = match resolve_canister_id(&req) {
        Some(canister_id) => canister_id,
        None => {
            return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Could not find Canister ID from Request.")));
        }
    };

    let canister = ic_utils::interfaces::HttpRequestCanister::create(&agent, canister_id.clone());

    let method = req.method().to_string();
    let uri = req.uri().to_string();
    let headers = req
        .headers()
        .into_iter()
        .filter_map(|(name, value)| {
            Some(HeaderField(
                name.to_string(),
                value.to_str().ok()?.to_string(),
            ))
        })
        .collect();
    let mut body = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        body.extend_from_slice(&item?);
    }
    let body = body.to_vec();

    debug!(
        logger,
        "Making call http_request to canister_id {}",
        canister_id.to_text()
    );

    match canister
        .http_request(method, uri, headers, body)
        .call()
        .await
    {
        Err(err) => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Details: {:?}", err))),
        Ok((http_response,)) => {
            if let Ok(status_code) = StatusCode::from_u16(http_response.status_code) {
                let mut builder = HttpResponse::build(status_code);
                for HeaderField(name, value) in http_response.headers {
                    builder.header(&name, value);
                }
                Ok(builder.body(http_response.body))
            } else {
                Ok(
                    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(format!(
                        "Invalid status code: {}",
                        http_response.status_code
                    )),
                )
            }
        }
    }
}

/// Run the webserver in the current thread.
pub fn run_webserver(
    logger: Logger,
    build_output_root: PathBuf,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
    providers: Vec<url::Url>,
    _serve_dir: PathBuf,
) -> DfxResult<Server> {
    const SHUTDOWN_WAIT_TIME: u64 = 60;
    info!(logger, "binding to: {:?}", bind);
    info!(
        logger,
        "replica(s): {}",
        providers
            .iter()
            .map(|x| x.clone().into_string())
            .collect::<Vec<String>>()
            .as_slice()
            .join(", ")
    );

    let forward_data = Arc::new(Mutex::new(ForwardActixData {
        providers,
        logger: logger.clone(),
        counter: 0,
    }));
    let candid_data = Arc::new(CandidData {
        build_output_root,
        network_descriptor,
    });
    let http_request_data = Arc::new(HttpRequestData {
        bind: bind.clone(),
        logger: logger.clone(),
    });

    let handler =
        HttpServer::new(move || {
            App::new()
                .data(
                    ClientBuilder::new()
                        .connector(Connector::new().limit(1).finish())
                        .finish(),
                )
                .data(forward_data.clone())
                .data(candid_data.clone())
                .data(http_request_data.clone())
                .wrap(
                    Cors::new()
                        .allowed_methods(vec!["POST"])
                        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                        .allowed_header(http::header::CONTENT_TYPE)
                        .send_wildcard()
                        .max_age(3600)
                        .finish(),
                )
                .wrap(middleware::Logger::default())
                .service(web::scope("/api").default_service(web::to(forward)))
                .service(web::resource("/_/candid").route(web::get().to(candid)))
                .service(web::resource("/_/").route(
                    web::get().to(|| HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)),
                ))
                .service(web::resource("/").route(web::get().to(http_request)))
            // .default_service(actix_files::Files::new("/", &serve_dir).index_file("index.html"))
        })
        .max_connections(1)
        .bind(bind)?
        // N.B. This is an arbitrary timeout for now.
        .shutdown_timeout(SHUTDOWN_WAIT_TIME)
        .run();

    Ok(handler)
}

pub fn webserver(
    logger: Logger,
    build_output_root: PathBuf,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
    clients_api_uri: Vec<url::Url>,
    serve_dir: &Path,
    inform_parent: Sender<Server>,
) -> DfxResult<std::thread::JoinHandle<()>> {
    // Verify that we cannot bind to a port that we forward to.
    let bound_port = bind.port();
    let bind_and_forward_on_same_port = clients_api_uri.iter().any(|url| {
        Some(bound_port) == url.port()
            && match url.host_str() {
                Some(h) => h == "localhost" || h == "::1" || h == "127.0.0.1",
                None => true,
            }
    });
    if bind_and_forward_on_same_port {
        return Err(error_unknown!(
            "Cannot forward API calls to the same bootstrap server."
        ));
    }

    std::thread::Builder::new()
        .name("Frontend".into())
        .spawn({
            let serve_dir = serve_dir.to_path_buf();
            move || {
                let _sys = System::new("dfx-frontend-http-server");
                let server = run_webserver(
                    logger,
                    build_output_root,
                    network_descriptor,
                    bind,
                    clients_api_uri,
                    serve_dir,
                )
                .unwrap();

                // Warning: Note that HttpServer provides its own signal
                // handler. That means if we provide signal handling beyond basic
                // we need to either as normal "re-signal" or disable_signals().
                let _ = inform_parent.send(server);
            }
        })
        .map_err(DfxError::from)
}
