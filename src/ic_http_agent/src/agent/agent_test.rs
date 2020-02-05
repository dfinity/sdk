use crate::agent::replica_api::{QueryResponseReply, ReadResponse, SubmitRequest};
use crate::agent::response::RequestStatusResponse;
use crate::{Agent, AgentConfig, AgentError, Blob, CanisterId, Waiter};
use mockito::mock;
use std::time::Duration;

#[test]
fn query() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = ReadResponse::Replied {
        reply: Some(QueryResponseReply { arg: blob.clone() }),
    };

    let read_mock = mock("POST", "/api/v1/read")
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
            .query(&CanisterId::from_bytes(&[1u8]), "main", &Blob(vec![]))
            .await
    });

    read_mock.assert();

    assert_eq!(result?, Some(blob));

    Ok(())
}

#[test]
fn query_error() -> Result<(), AgentError> {
    let read_mock = mock("POST", "/api/v1/read").with_status(500).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let result: Result<Option<Blob>, AgentError> = runtime.block_on(async {
        agent
            .query(&CanisterId::from_bytes(&[2u8]), "greet", &Blob::empty())
            .await
    });

    read_mock.assert();

    assert!(result.is_err());

    Ok(())
}

#[test]
fn query_rejected() -> Result<(), AgentError> {
    let response: ReadResponse<u8> = ReadResponse::Rejected {
        reject_code: 1234,
        reject_message: "Rejected Message".to_string(),
    };

    let read_mock = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");

    let result: Result<Option<Blob>, AgentError> = runtime.block_on(async {
        agent
            .query(&CanisterId::from_bytes(&[3u8]), "greet", &Blob::empty())
            .await
    });

    read_mock.assert();

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
fn call() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = ReadResponse::Replied {
        reply: Some(QueryResponseReply { arg: blob.clone() }),
    };

    let submit_mock = mock("POST", "/api/v1/submit").with_status(200).create();
    let status_mock = mock("POST", "/api/v1/read")
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
            .call(&CanisterId::from_bytes(&[4u8]), "greet", &Blob::empty())
            .await?;
        agent.request_status(&request_id).await
    });

    submit_mock.assert();
    status_mock.assert();

    assert_eq!(
        result?,
        RequestStatusResponse::Replied { reply: Some(blob) }
    );

    Ok(())
}

#[test]
fn call_error() -> Result<(), AgentError> {
    let submit_mock = mock("POST", "/api/v1/submit").with_status(500).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .call(&CanisterId::from_bytes(&[5u8]), "greet", &Blob::empty())
            .await
    });

    submit_mock.assert();

    assert!(result.is_err());

    Ok(())
}

#[test]
fn call_rejected() -> Result<(), AgentError> {
    let response: ReadResponse<u8> = ReadResponse::Rejected {
        reject_code: 1234,
        reject_message: "Rejected Message".to_string(),
    };

    let submit_mock = mock("POST", "/api/v1/submit").with_status(200).create();
    let status_mock = mock("POST", "/api/v1/read")
        .with_status(200)
        .with_header("content-type", "application/cbor")
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result: Result<Option<Blob>, AgentError> = runtime.block_on(async {
        let request_id = agent
            .call(&CanisterId::from_bytes(&[6u8]), "greet", &Blob::empty())
            .await?;
        agent
            .request_status_and_wait(&request_id, Waiter::timeout(Duration::from_millis(1)))
            .await
    });

    submit_mock.assert();
    status_mock.assert();

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
fn install() -> Result<(), AgentError> {
    let canister_id = CanisterId::from_bytes(&[5u8]);
    let module = Blob::from(&[1, 2]);

    let blob = Blob(Vec::from("Hello World"));
    let response = ReadResponse::Replied {
        reply: Some(QueryResponseReply { arg: blob.clone() }),
    };

    let submit_mock = mock("POST", "/api/v1/submit").with_status(200).create();
    let status_mock = mock("POST", "/api/v1/read")
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
        let request_id = agent.install(&canister_id, &module, &Blob::empty()).await?;
        agent.request_status(&request_id).await
    });

    submit_mock.assert();
    status_mock.assert();

    assert_eq!(
        result?,
        RequestStatusResponse::Replied { reply: Some(blob) }
    );

    Ok(())
}

#[test]
fn ping() -> Result<(), AgentError> {
    let read_mock = mock("GET", "/api/v1/read").with_status(200).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async { agent.ping(Waiter::instant()).await });

    read_mock.assert();

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn ping_okay() -> Result<(), AgentError> {
    let read_mock = mock("GET", "/api/v1/read").with_status(405).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async { agent.ping(Waiter::instant()).await });

    read_mock.assert();

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn ping_error() -> Result<(), AgentError> {
    let read_mock = mock("GET", "/api/v1/read")
        .expect(2)
        .with_status(500)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .ping(
                Waiter::builder()
                    .throttle(Duration::from_millis(4))
                    .timeout(Duration::from_millis(6))
                    .build(),
            )
            .await
    });

    read_mock.assert();

    assert!(result.is_err());

    Ok(())
}
