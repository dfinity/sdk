use futures::future::Future;

fn main() {
    let client = dfx::Client::new();
    let query = dfx::query(
        client,
        dfx::CanisterQueryCall {
            canister_id: 42,
            method_name: "dfn_msg greet".to_string(),
            arg: None,
        },
    )
    .map(|r| {
        match r {
            dfx::QueryResponse::Replied { reply: dfx::QueryResponseReply{ arg: bytes}} =>
                println!("{}", String::from_utf8_lossy(&bytes)),
            dfx::QueryResponse::Rejected => panic!("oops!"),
        }
    })
    .map_err(|e| {
        println!("{:#?}", e);
        ::std::process::exit(1);
    });
    tokio::run(query);
}
