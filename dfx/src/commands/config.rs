use crate::lib::env::ProjectConfigEnv;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::value::Value;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("config")
        .about(UserMessage::ConfigureOptions.to_str())
        .arg(
            Arg::with_name("config_path")
                .help(UserMessage::OptionName.to_str())
                .required(true),
        )
        .arg(Arg::with_name("value").help(UserMessage::OptionValue.to_str()))
}

pub fn exec<T: ProjectConfigEnv>(env: &T, args: &ArgMatches<'_>) -> DfxResult {
    // Cannot use the `env` variable as we need a mutable copy.
    let mut config = env
        .get_config()
        .ok_or_else(DfxError::CommandMustBeRunInAProject)?
        .clone();

    let config_path = args.value_of("config_path").unwrap();

    // We replace `.` with `/` so the user can use `path.value.field` instead of forcing him
    // to use `path/value/field`. Since none of our keys have slashes or tildes in them it
    // won't be a problem.
    let mut config_path = config_path.replace(".", "/");
    // We change config path to starts with a `/` if it doesn't already. This is because
    // JSON pointers can be relative, but we don't have a place to start if is it.
    if !config_path.starts_with('/') {
        config_path.insert(0, '/');
    }

    if let Some(arg_value) = args.value_of("value") {
        // Try to parse the type of the value (which is a string from the arguments) as
        // JSON. By default we will just assume the type is string (if all parsing fails).
        let value = serde_json::from_str::<Value>(arg_value)
            .unwrap_or_else(|_| Value::String(arg_value.to_owned()));

        *config
            .get_mut_json()
            .pointer_mut(config_path.as_str())
            .unwrap() = value;

        config.save()
    } else if let Some(value) = config.get_json().pointer(config_path.as_str()) {
        println!("{}", value);
        Ok(())
    } else {
        Err(DfxError::ConfigPathDoesNotExist(config_path))
    }
}
