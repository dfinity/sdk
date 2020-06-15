use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
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
    // Need storage for AgentEnvironment ownership.
    let mut _agent_env: Option<AgentEnvironment<'_>> = None;
    let env = if args.is_present("provider") {
        _agent_env = Some(AgentEnvironment::new(
            env,
            args.value_of("provider").expect("Could not find provider."),
        ));
        _agent_env.as_ref().unwrap()
    } else {
        env
    };

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
