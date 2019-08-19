extern crate futures;
extern crate tokio;

use futures::future::Future;

fn main() {
    let client = reqwest::async::Client::new();
    let query = dfx::query(client, dfx::CanisterQueryCall {
        canister_id: 0,
        method_name: "main".to_string(),
        arg: None,
    })
    .map(|r| {
        println!("{}", r.reply);
    })
    .map_err(|e| {
        println!("{:#?}", e);
        ::std::process::exit(1);
    });
    tokio::run(query);
}
