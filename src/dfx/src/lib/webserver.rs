// use crate::error_unknown;
use crate::lib::error::DfxResult;
use crate::lib::locations::canister_did_location;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::check_candid_file;

//use actix::System;
use actix_cors::Cors;
use actix_server::Server;
use actix_web::client::{ClientBuilder, Connector};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use actix_web::{
    http, middleware, web, App, Error, HttpResponse, HttpServer,
};
use anyhow::anyhow;
//use candid::parser::value::IDLValue;
//use crossbeam::channel::Sender;
//use futures::StreamExt;
//use ic_agent::{Agent, AgentError};
//use ic_types::Principal;
//use ic_utils::call::SyncCall;
//use ic_utils::interfaces::http_request::HeaderField;
//use ic_utils::interfaces::http_request::StreamingStrategy::Callback;
//use ic_utils::interfaces::HttpRequestCanister;
//use ic_utils::Canister;
use serde::Deserialize;
use slog::{info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;
//use std::str::FromStr;
use std::sync::{Arc, Mutex};
use url::Url;

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

// fn resolve_canister_id_from_hostname(hostname: &str) -> Option<Principal> {
//     let url = Uri::from_str(hostname).ok()?;
//
//     // Check if it's localhost or ic0.
//     match url.host()?.split('.').collect::<Vec<&str>>().as_slice() {
//         [.., maybe_canister_id, "localhost"] | [.., maybe_canister_id, "ic0", "app"] => {
//             if let Ok(canister_id) = Principal::from_text(maybe_canister_id) {
//                 return Some(canister_id);
//             }
//         }
//         _ => {}
//     };
//
//     None
// }
//
// fn resolve_canister_id_from_query(url: &Uri) -> Option<Principal> {
//     let (_, canister_id) = url::form_urlencoded::parse(url.query()?.as_bytes())
//         .find(|(name, _)| name == "canisterId")?;
//     Principal::from_text(canister_id.as_ref()).ok()
// }
//
// fn resolve_canister_id(request: &HttpRequest) -> Option<Principal> {
//     // Look for subdomains if there's a host header.
//     if let Some(host_header) = request.headers().get("Host") {
//         if let Ok(host) = host_header.to_str() {
//             if let Some(canister_id) = resolve_canister_id_from_hostname(host) {
//                 return Some(canister_id);
//             }
//         }
//     }
//
//     // Look into the URI.
//     if let Some(canister_id) = resolve_canister_id_from_query(request.uri()) {
//         return Some(canister_id);
//     }
//
//     // Look into the request by header.
//     if let Some(referer_header) = request.headers().get("referer") {
//         if let Ok(referer) = referer_header.to_str() {
//             if let Ok(referer_uri) = Uri::from_str(referer) {
//                 if let Some(canister_id) = resolve_canister_id_from_query(&referer_uri) {
//                     return Some(canister_id);
//                 }
//             }
//         }
//     }
//
//     None
// }

/// Run the webserver in the current thread.
pub fn run_webserver(
    logger: Logger,
    build_output_root: PathBuf,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
    providers: Vec<url::Url>,
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
        bind,
        logger: logger.clone(),
    });

    let handler =
        HttpServer::new(move || {
            App::new()
                .data(
                    ClientBuilder::new()
                        .connector(Connector::new().limit(10).finish())
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
                //.service(web::scope("/api").default_service(web::to(forward)))
                .service(web::resource("/_/candid").route(web::get().to(candid)))
                .service(web::resource("/_/").route(
                    web::get().to(|| HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)),
                ))
                //.default_service(web::get().to(http_request))
                .default_service(web::get().to(|| HttpResponse::build(StatusCode::NOT_FOUND)))
        })
        .max_connections(10)
        .bind(bind)?
        // N.B. This is an arbitrary timeout for now.
        .shutdown_timeout(SHUTDOWN_WAIT_TIME)
        .run();

    Ok(handler)
}

