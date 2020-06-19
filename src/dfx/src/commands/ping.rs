use crate::config::dfinity::ConfigNetwork;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::provider::command_line_provider_to_url;
use crate::lib::waiter::create_waiter;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_cbor::Value;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("ping")
        .about(UserMessage::Ping.to_str())
        .arg(
            Arg::with_name("provider")
                .help("The provider to use.")
                .takes_value(true),
        )
}

pub fn cbor_to_json(cbor: &Value) -> DfxResult<serde_json::Value> {
    Ok(match cbor {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => {
            serde_json::Value::Number(serde_json::Number::from_f64(*i as f64).unwrap())
        }
        Value::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap()),
        Value::Bytes(_) => {
            return Err(DfxError::Unknown(
                "Cannot serialize Bytes strings to JSON.".to_string(),
            ))
        }
        Value::Text(s) => serde_json::Value::String(s.to_owned()),
        Value::Array(a) => {
            let mut vec = Vec::new();
            for i in a {
                vec.push(cbor_to_json(i)?);
            }
            serde_json::Value::Array(vec)
        }
        Value::Map(m) => {
            let mut map = serde_json::Map::new();
            for (k, v) in m {
                let k = match k {
                    Value::Text(s) => s.clone(),
                    _ => {
                        return Err(DfxError::Unknown(
                            "Cannot serialize non-string keys to JSON.".to_string(),
                        ))
                    }
                };
                map.insert(k, cbor_to_json(v)?);
            }
            serde_json::Value::Object(map)
        }
        _ => return Err(DfxError::Unknown("Invalid CBOR type.".to_string())),
    })
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let config = config.as_ref().get_config();

    // For ping, "provider" could either be a URL or a network name.
    // If not passed, we default to the "local" network.
    let provider = args.value_of("provider").unwrap_or("local");
    let agent_url = match config.get_network(&provider) {
        Some(ConfigNetwork::ConfigNetworkProvider(network_provider)) => {
            match network_provider.providers.first() {
                Some(url) => Ok(url.clone()),
                None => Err(DfxError::ComputeNetworkHasNoProviders(provider.to_string())),
            }?
        }
        Some(ConfigNetwork::ConfigLocalProvider(local_provider)) => {
            format!("http://{}", local_provider.bind)
        }
        None => command_line_provider_to_url(&provider)?,
    };

    let env = AgentEnvironment::new(env, &agent_url);

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(agent.ping(create_waiter()))?;

    if let Value::Map(_) = &result {
        let json = cbor_to_json(&result)?;
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        slog::error!(
            env.get_logger(),
            "Invalid CBOR value. Was expected map, got {:?}",
            result
        );
    }

    Ok(())
}
