#![cfg(test)]
use crate::agent::agent_impl::{AgentConfig, QueryResponseReply, ReadResponse};
use crate::agent::response::RequestStatusResponse;
use crate::{Agent, AgentError, Blob, CanisterId, Waiter};
use mockito::mock;
use serde_idl::Encode;
use std::time::Duration;

#[test]
fn query_blob() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = ReadResponse::Replied {
        reply: Some(QueryResponseReply { arg: blob.clone() }),
    };

    let _m = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
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
        reply: Some(QueryResponseReply {
            arg: Blob::from(Encode!(&vec)),
        }),
    };

    let _m = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
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
fn query_idl_error() -> Result<(), AgentError> {
    let _m = mock("POST", "/api/v1/read").with_status(500).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let result: Result<String, AgentError> = runtime.block_on(async {
        agent
            .query(&CanisterId::from(1234), "greet", &"World".to_string())
            .await
    });

    _m.assert();

    assert!(result.is_err());

    Ok(())
}

#[test]
fn query_idl_rejected() -> Result<(), AgentError> {
    let response: ReadResponse<u8> = ReadResponse::Rejected {
        reject_code: 1234,
        reject_message: "Rejected Message".to_string(),
    };

    let _m = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
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

#[test]
fn call_idl() -> Result<(), AgentError> {
    let vec = "Hello World".to_string();
    let response = ReadResponse::Replied {
        reply: Some(QueryResponseReply {
            arg: Blob::from(Encode!(&vec)),
        }),
    };

    let _c = mock("POST", "/api/v1/submit").with_status(200).create();
    let _status = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result: Result<String, AgentError> = runtime.block_on(async {
        let request_id = agent
            .call(&CanisterId::from(1234), "greet", &"World".to_string())
            .await?;
        agent
            .request_status_and_wait::<String>(
                &request_id,
                Waiter::throttle_and_timeout(Duration::from_secs(100), Duration::from_millis(10)),
            )
            .await
    });

    _c.assert();
    _status.assert();

    assert_eq!(result?, vec);

    Ok(())
}

#[test]
fn call_blob() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = ReadResponse::Replied {
        reply: Some(QueryResponseReply { arg: blob.clone() }),
    };

    let _c = mock("POST", "/api/v1/submit").with_status(200).create();
    let _status = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        let request_id = agent
            .call_blob(&CanisterId::from(1234), "greet", &Blob::empty())
            .await?;
        agent.request_status_blob(&request_id).await
    });

    _c.assert();
    _status.assert();

    assert_eq!(result?, RequestStatusResponse::Replied { reply: blob });

    Ok(())
}

#[test]
fn call_blob_error() -> Result<(), AgentError> {
    let _c = mock("POST", "/api/v1/submit").with_status(500).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .call(&CanisterId::from(1234), "greet", &"World".to_string())
            .await
    });

    _c.assert();

    assert!(result.is_err());

    Ok(())
}

#[test]
fn call_blob_rejected() -> Result<(), AgentError> {
    let response: ReadResponse<u8> = ReadResponse::Rejected {
        reject_code: 1234,
        reject_message: "Rejected Message".to_string(),
    };

    let _c = mock("POST", "/api/v1/submit").with_status(200).create();
    let _status = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result: Result<Blob, AgentError> = runtime.block_on(async {
        let request_id = agent
            .call_blob(&CanisterId::from(1234), "greet", &Blob::empty())
            .await?;
        agent
            .request_status_and_wait(
                &request_id,
                Waiter::throttle_and_timeout(Duration::from_secs(100), Duration::from_millis(10)),
            )
            .await
    });

    _c.assert();
    _status.assert();

    match result {
        Err(AgentError::ClientError(code, msg)) => {
            assert_eq!(code, 1234);
            assert_eq!(msg, "Rejected Message");
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[test]
fn ping() -> Result<(), AgentError> {
    let _m = mock("GET", "/api/v1/read").with_status(200).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .ping(Waiter::throttle_and_timeout(
                Duration::from_millis(100),
                Duration::from_secs(3),
            ))
            .await
    });

    _m.assert();

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn ping_okay() -> Result<(), AgentError> {
    let _m = mock("GET", "/api/v1/read").with_status(405).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .ping(Waiter::throttle_and_timeout(
                Duration::from_millis(100),
                Duration::from_secs(3),
            ))
            .await
    });

    _m.assert();

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn ping_error() -> Result<(), AgentError> {
    let _m = mock("GET", "/api/v1/read")
        .expect(3)
        .with_status(500)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .ping(Waiter::throttle_and_timeout(
                Duration::from_millis(40),
                Duration::from_millis(60),
            ))
            .await
    });

    _m.assert();

    assert!(result.is_err());

    Ok(())
}
