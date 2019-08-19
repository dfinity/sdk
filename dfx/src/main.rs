fn main() {
    let client = reqwest::Client::new();
    let res = dfx::query(client, dfx::CanisterQueryCall {
        canister_id: 0,
        method_name: "main".to_string(),
        arg: None,
    });
    match res {
        Ok(r) => println!("{}", r.reply),
        Err(e) => {
            println!("{:#?}", e);
            ::std::process::exit(1);
        },
    }
}
