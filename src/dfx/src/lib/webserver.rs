use actix::dev::Stream;
use actix::System;
use actix_cors::Cors;
use actix_server::Server;
use actix_web::client::Client;
use actix_web::{
    http, middleware, web, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer,
};
use crossbeam::channel::Sender;
use futures::Future;
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

fn forward(
    req: HttpRequest,
    payload: web::Payload,
    client: web::Data<Client>,
    actix_data: web::Data<Arc<Mutex<ForwardActixData>>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
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
    payload
        .map_err(Error::from)
        .fold(web::BytesMut::new(), move |mut body, chunk| {
            body.extend_from_slice(&chunk);
            Ok::<_, Error>(body)
        })
        .and_then(move |req_body| {
            // We streamed the whole body in memory. Let's log some informations.
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

            forwarded_req
                .send_body(req_body)
                .map_err(Error::from)
                .and_then(|mut res| {
                    res.take_payload()
                        .map_err(Error::from)
                        .fold(web::BytesMut::new(), move |mut body, chunk| {
                            body.extend_from_slice(&chunk);
                            Ok::<_, Error>(body)
                        })
                        .and_then(move |res_body| {
                            let mut client_resp = HttpResponse::build(res.status());
                            for (header_name, header_value) in res
                                .headers()
                                .iter()
                                .filter(|(h, _)| *h != "connection" && *h != "content-length")
                            {
                                client_resp.header(header_name.clone(), header_value.clone());
                            }

                            debug!(
                                logger,
                                "Response ({}) with status code {}",
                                indicatif::HumanBytes(res_body.len() as u64),
                                res.status().as_u16()
                            );
                            trace!(logger, "  type  {}", res.content_type());
                            trace!(logger, "  body  {}", hex::encode(&res_body));

                            client_resp.body(res_body)
                        })
                })
        })
}

/// Run the webserver in the current thread.
pub fn run_webserver(
    logger: Logger,
    bind: SocketAddr,
    providers: Vec<url::Url>,
    serve_dir: PathBuf,
    inform_parent: Sender<Server>,
) -> Result<(), std::io::Error> {
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

    let _sys = System::new("dfx-frontend-http-server");
    let forward_data = Arc::new(Mutex::new(ForwardActixData {
        providers,
        logger: logger.clone(),
        counter: 0,
    }));

    let handler = HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(forward_data.clone())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["POST"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .send_wildcard()
                    .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .service(web::scope("/api").default_service(web::to_async(forward)))
            .default_service(actix_files::Files::new("/", &serve_dir).index_file("index.html"))
    })
    .bind(bind)?
    // N.B. This is an arbitrary timeout for now.
    .shutdown_timeout(SHUTDOWN_WAIT_TIME)
    .system_exit()
    .start();

    // Warning: Note that HttpServer provides its own signal
    // handler. That means if we provide signal handling beyond basic
    // we need to either as normal "re-signal" or disable_signals().
    let _ = inform_parent.send(handler);

    Ok(())
}

pub fn webserver(
    logger: Logger,
    bind: SocketAddr,
    clients_api_uri: Vec<url::Url>,
    serve_dir: &Path,
    inform_parent: Sender<Server>,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new().name("Frontend".into()).spawn({
        let serve_dir = serve_dir.to_path_buf();
        move || run_webserver(logger, bind, clients_api_uri, serve_dir, inform_parent).unwrap()
    })
}
