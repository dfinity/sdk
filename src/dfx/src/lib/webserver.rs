use crate::lib::error::DfxResult;
use crate::lib::locations::canister_did_location;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::check_candid_file;

use actix_cors::Cors;
use actix_server::Server;
use actix_web::client::{ClientBuilder, Connector};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer};
use anyhow::anyhow;
use serde::Deserialize;
use slog::{info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

struct CandidData {
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
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

/// Run the webserver in the current thread.
pub fn run_webserver(
    logger: Logger,
    build_output_root: PathBuf,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
) -> DfxResult<Server> {
    const SHUTDOWN_WAIT_TIME: u64 = 60;
    info!(logger, "binding to: {:?}", bind);
    let candid_data = Arc::new(CandidData {
        build_output_root,
        network_descriptor,
    });

    let handler =
        HttpServer::new(move || {
            App::new()
                .data(
                    ClientBuilder::new()
                        .connector(Connector::new().limit(1).finish())
                        .finish(),
                )
                .data(candid_data.clone())
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
                .service(web::resource("/_/candid").route(web::get().to(candid)))
                .service(web::resource("/_/").route(
                    web::get().to(|| HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)),
                ))
                .default_service(web::get().to(|| HttpResponse::build(StatusCode::NOT_FOUND)))
        })
        .max_connections(1)
        .bind(bind)?
        // N.B. This is an arbitrary timeout for now.
        .shutdown_timeout(SHUTDOWN_WAIT_TIME)
        .run();

    Ok(handler)
}
