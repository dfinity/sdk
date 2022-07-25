use crate::config::dfinity::Config;
use crate::lib::error::DfxResult;
use crate::lib::locations::canister_did_location;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::check_candid_file;

use actix_cors::Cors;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer};
use anyhow::{anyhow, Context};
use fn_error_context::context;
use serde::Deserialize;
use slog::{info, Logger};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

struct CandidData {
    pub build_output_root: PathBuf,
    pub network_descriptor: NetworkDescriptor,
    pub config: Arc<Config>,
    pub project_temp_dir: PathBuf,
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
    data: web::Data<CandidData>,
) -> Result<HttpResponse, Error> {
    let id = info.canister_id;
    let network_descriptor = &data.network_descriptor;
    let store = CanisterIdStore::new(
        network_descriptor,
        Some(data.config.clone()),
        &data.project_temp_dir,
    )
    .map_err(ErrorInternalServerError)?;

    let candid_path = store
        .get_name(&id)
        .map(|canister_name| canister_did_location(&data.build_output_root, canister_name))
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

/// Run the webserver in another thread.
#[context("Failed to run webserver.")]
pub fn run_webserver(
    logger: Logger,
    build_output_root: PathBuf,
    network_descriptor: NetworkDescriptor,
    config: Arc<Config>,
    project_temp_dir: PathBuf,
    bind: SocketAddr,
) -> DfxResult {
    const SHUTDOWN_WAIT_TIME: u64 = 60;
    info!(logger, "binding to: {:?}", bind);
    let candid_data = web::Data::new(CandidData {
        build_output_root,
        network_descriptor,
        config,
        project_temp_dir,
    });

    let handler =
        HttpServer::new(move || {
            App::new()
                .app_data(candid_data.clone())
                .wrap(
                    Cors::default()
                        .allowed_methods(vec!["POST"])
                        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                        .allowed_header(http::header::CONTENT_TYPE)
                        .send_wildcard()
                        .max_age(3600),
                )
                .wrap(middleware::Logger::default())
                .service(web::resource("/_/candid").route(web::get().to(candid)))
                .service(web::resource("/_/").route(
                    web::get().to(|| HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)),
                ))
                .default_service(web::get().to(|| HttpResponse::build(StatusCode::NOT_FOUND)))
        })
        .bind(bind)
        .with_context(|| format!("Failed to bind HTTP server to {:?}.", bind))?
        // N.B. This is an arbitrary timeout for now.
        .shutdown_timeout(SHUTDOWN_WAIT_TIME)
        .run();
    thread::spawn(|| {
        actix::run(async {
            handler.await.unwrap();
        })
    });
    Ok(())
}
