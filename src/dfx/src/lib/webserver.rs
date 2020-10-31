use crate::lib::error::{DfxError, DfxResult};
use crate::lib::locations::canister_did_location;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::check_candid_file;
use actix::System;
use actix_cors::Cors;
use actix_server::Server;
use actix_web::client::Client;
use actix_web::{
    http, middleware, web, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer,
};
use crossbeam::channel::Sender;
use futures::StreamExt;
use serde::Deserialize;
use slog::{debug, info, trace, Logger};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
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

#[derive(Deserialize)]
enum Format {
    #[serde(rename = "js")]
    Javascript,
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
) -> DfxResult<HttpResponse> {
    let id = info.canister_id;
    let network_descriptor = &data.network_descriptor;
    let store = CanisterIdStore::for_network(&network_descriptor)?;
    let candid_path = store
        .get_name(&id)
        .map(|canister_name| canister_did_location(&data.build_output_root, &canister_name))
        .ok_or_else(|| {
            DfxError::CouldNotFindCanisterNameForNetwork(
                id.to_string(),
                network_descriptor.name.clone(),
            )
        })?
        .canonicalize()
        .map_err(|_e| DfxError::Unknown("cannot find candid file".to_string()))?;

    let content = match info.format {
        None => std::fs::read_to_string(candid_path)?,
        Some(Format::Javascript) => {
            let (env, ty) = check_candid_file(&candid_path)?;
            candid::bindings::javascript::compile(&env, &ty)
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

    // TODO(hansl): move this all to async/await. Jeez....
    // (PS: the reason I don't do this yet is moving actix to async/await is a bit more
    //      involved than replacing this single function)
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

/// Run the webserver in the current thread.
pub fn run_webserver(
    logger: Logger,
    build_output_root: PathBuf,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
    providers: Vec<url::Url>,
    serve_dir: PathBuf,
) -> DfxResult<Server> {
    info!(logger, "binding to: {:?}", bind);

    const SHUTDOWN_WAIT_TIME: u64 = 60;

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

    let handler = HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(forward_data.clone())
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
            .service(web::scope("/api").default_service(web::to(forward)))
            .service(web::resource("/_/candid").route(web::get().to(candid)))
            .default_service(actix_files::Files::new("/", &serve_dir).index_file("index.html"))
    })
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
        return Err(DfxError::Unknown(
            "Cannot forward API calls to the same bootstrap server.".to_string(),
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
