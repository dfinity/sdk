#[cfg(test)]
mod tests {
    use futures::future::Future;
    use mockito::mock;

    #[test]
    fn query_hello_world() {
        let _ = env_logger::try_init();

        let response = dfx::Response {
            status: dfx::Status::Replied,
            reply: Some("Hello World"),
            reject_code: None,
            reject_message: None,
        };

        let _m = mock("POST", "/api/v1/read")
            .with_status(200)
            .with_header("content-type", "application/cbor")
            .with_body(serde_cbor::to_vec(&response).unwrap())
            .create();

        let client = dfx::Client::new();

        let query = dfx::query(client, dfx::CanisterQueryCall {
            canister_id: 0,
            method_name: "main".to_string(),
            arg: None,
        })
        .map(|r| {
            println!("{}", r.reply.unwrap());
        })
        .map_err(|e| {
            println!("{:#?}", e);
        });

        tokio::run(query);

        _m.assert();
    }
}
