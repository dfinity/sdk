use crate::agent::replica_api::{CallReply, QueryResponse};
use crate::agent::response::{Replied, RequestStatusResponse};
use crate::{Agent, AgentConfig, AgentError, Blob, CanisterId};
use delay::Delay;
use mockito::mock;
use std::collections::BTreeMap;
use std::time::Duration;

#[test]
fn query() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = QueryResponse::Replied {
        reply: CallReply { arg: blob.clone() },
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

    assert_eq!(result?, blob);

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

    let result: Result<Blob, AgentError> = runtime.block_on(async {
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
    let response: QueryResponse = QueryResponse::Rejected {
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

    let result: Result<Blob, AgentError> = runtime.block_on(async {
        agent
            .query(&CanisterId::from_bytes(&[3u8]), "greet", &Blob::empty())
            .await
    });

    read_mock.assert();

    match result {
        Err(AgentError::ReplicaError {
            reject_code: code,
            reject_message: msg,
        }) => {
            assert_eq!(code, 1234);
            assert_eq!(msg, "Rejected Message");
        }
        result => unreachable!("{:?}", result),
    }

    Ok(())
}

#[test]
fn call() -> Result<(), AgentError> {
    let blob = Blob(Vec::from("Hello World"));
    let response = QueryResponse::Replied {
        reply: CallReply { arg: blob.clone() },
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
            .call_raw(&CanisterId::from_bytes(&[4u8]), "greet", &Blob::empty())
            .await?;
        agent.request_status_raw(&request_id).await
    });

    submit_mock.assert();
    status_mock.assert();

    assert_eq!(
        result?,
        RequestStatusResponse::Replied {
            reply: Replied::CallReplied(blob)
        }
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
    let response: QueryResponse = QueryResponse::Rejected {
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
    let result: Result<Replied, AgentError> = runtime.block_on(async {
        let request_id = agent
            .call_raw(&CanisterId::from_bytes(&[6u8]), "greet", &Blob::empty())
            .await?;
        agent
            .request_status_and_wait(&request_id, Delay::timeout(Duration::from_millis(100)))
            .await
    });

    match result {
        Err(AgentError::ReplicaError {
            reject_code: code,
            reject_message: msg,
        }) => {
            assert_eq!(code, 1234);
            assert_eq!(msg, "Rejected Message");
        }
        result => unreachable!("{:?}", result),
    }

    submit_mock.assert();
    status_mock.assert();

    Ok(())
}

#[test]
fn ping() -> Result<(), AgentError> {
    let response = serde_cbor::Value::Map(BTreeMap::new());
    let read_mock = mock("GET", "/api/v1/status")
        .with_status(200)
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async { agent.ping(Delay::instant()).await });

    read_mock.assert();

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn ping_okay() -> Result<(), AgentError> {
    let response = serde_cbor::Value::Map(BTreeMap::new());
    let read_mock = mock("GET", "/api/v1/status")
        .with_status(200)
        .with_body(serde_cbor::to_vec(&response)?)
        .create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(agent.ping(Delay::instant()));

    read_mock.assert();

    assert!(result.is_ok());

    Ok(())
}

#[test]
// test that the agent (re)tries to reach the server.
// We spawn an agent that waits 400ms between requests, and times out after 600ms. The agent is
// expected to hit the server at ~ 0ms and ~ 400 ms, and then shut down at 600ms, so we check that
// the server got two requests.
fn ping_error() -> Result<(), AgentError> {
    // This mock is never asserted as we don't know (nor do we need to know) how many times
    // it is called.
    let _read_mock = mock("GET", "/api/v1/status").with_status(500).create();

    let agent = Agent::new(AgentConfig {
        url: &mockito::server_url(),
        ..AgentConfig::default()
    })?;
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(async {
        agent
            .ping(
                Delay::builder()
                    .throttle(Duration::from_millis(4))
                    .timeout(Duration::from_millis(6))
                    .build(),
            )
            .await
    });

    assert!(result.is_err());

    Ok(())
}
