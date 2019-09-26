use actix_web::client::Client;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use futures::Future;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use url::Url;

fn forward(
    req: HttpRequest,
    payload: web::Payload,
    url: web::Data<Url>,
    client: web::Data<Client>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    forwarded_req
        .send_stream(payload)
        .map_err(Error::from)
        .map(|res| {
            let mut client_resp = HttpResponse::build(res.status());
            for (header_name, header_value) in res
                .headers()
                .iter()
                .filter(|(h, _)| *h != "connection" && *h != "content-length")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }
            client_resp.streaming(res)
        })
}

/// Run the webserver in the current thread.
fn run_webserver(
    bind: SocketAddr,
    client_api_uri: url::Url,
    serve_dir: PathBuf,
) -> Result<(), std::io::Error> {
    eprintln!("binding to: {:?}", bind);
    eprintln!("client: {:?}", client_api_uri);

    HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(client_api_uri.clone())
            .wrap(middleware::Logger::default())
            .service(web::scope(client_api_uri.path()).default_service(web::to_async(forward)))
            .default_service(actix_files::Files::new("/", &serve_dir).index_file("index.html"))
    })
    .bind(bind)?
    .system_exit()
    .start();

    Ok(())
}

pub fn webserver(
    bind: SocketAddr,
    client_api_uri: url::Url,
    serve_dir: &Path,
) -> std::thread::JoinHandle<()> {
    let serve_dir = PathBuf::from(serve_dir);
    std::thread::spawn(move || run_webserver(bind, client_api_uri, serve_dir).unwrap())
}
