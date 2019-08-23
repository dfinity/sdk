use dfx::*;
use futures::future::Future;

fn main() {
    let client = Client::new(ClientConfig {
        url: "http://localhost:8080".to_string(),
        // url: "http://10.129.10.139:8080".to_string(),
    });
    let query = query(
        client,
        CanisterQueryCall {
            canister_id: 42,
            method_name: "dfn_msg greet".to_string(),
            arg: None,
        },
    )
    .map(|r| match r {
        Response::Accepted => println!("Accepted"),
        Response::Replied {
            reply: QueryResponseReply { arg: Blob(blob) },
        } => println!("{}", String::from_utf8_lossy(&blob)),
        Response::Rejected {
            reject_code,
            reject_message,
        } => panic!(format!("{:?}, {}", reject_code, reject_message)),
        Response::Unknown => panic!("Unknown response"),
    })
    .map_err(|e| {
        println!("{:#?}", e);
        ::std::process::exit(1);
    });
    tokio::run(query);
}
