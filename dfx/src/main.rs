use futures::future::Future;
use std::sync::Arc;
use std::cell::RefCell;


fn main() {
    let client = Arc::new(dfx::AsyncClient::new());
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
