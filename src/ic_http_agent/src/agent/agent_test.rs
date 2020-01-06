#![cfg(test)]
use crate::agent::agent_impl::ReadResponse;
use crate::{Agent, AgentError, Blob, CanisterId};
use mockito::mock;
use serde_idl::Encode;

#[test]
fn query_blob() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = ReadResponse::Replied {
        reply: Some(blob.clone()),
    };

    let _m = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::with_url(mockito::server_url())?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .query_blob(&CanisterId::from(1), "main", &Blob(vec![]))
            .await
    });

    _m.assert();

    assert_eq!(result?, blob);

    Ok(())
}

#[test]
fn query_idl() -> Result<(), AgentError> {
    let vec = "Hello World".to_string();
    let response = ReadResponse::Replied {
        reply: Some(Blob::from(Encode!(&vec))),
    };

    let _m = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::with_url(mockito::server_url())?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let result: Result<String, AgentError> = runtime.block_on(async {
        agent
            .query(&CanisterId::from(1234), "greet", &"World".to_string())
            .await
    });

    _m.assert();

    assert_eq!(result?, vec);

    Ok(())
}

#[test]
fn query_idl_rejected() -> Result<(), AgentError> {
    let response: ReadResponse<String> = ReadResponse::Rejected {
        reject_code: 1234,
        reject_message: "Rejected Message".to_string(),
    };

    let _m = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::with_url(mockito::server_url())?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let result: Result<String, AgentError> = runtime.block_on(async {
        agent
            .query(&CanisterId::from(1234), "greet", &"World".to_string())
            .await
    });

    _m.assert();

    match result {
        Err(AgentError::ClientError(code, msg)) => {
            assert_eq!(code, 1234);
            assert_eq!(msg, "Rejected Message");
        }
        _ => unreachable!(),
    }

    Ok(())
}
