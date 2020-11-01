use crate::config::dfinity::Config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use serde_json::value::Value;

/// Configures project options for your currently-selected project.
#[derive(Clap)]
pub struct ConfigOpts {
    /// Specifies the name of the configuration option to set or read.
    /// Use the period delineated path to specify the option to set or read.
    /// If this is not mentioned, outputs the whole configuration.
    #[clap(long)]
    config_path: String,

    /// Specifies the new value to set.
    /// If you don't specify a value, the command displays the current value of the option from the configuration file.
    #[clap(long)]
    value: String,

    /// Specifies the format of the output. By default, the output format is JSON.
    #[clap(long, default_value("json"), possible_values(&["json", "text"]))]
    format: String,
}

pub fn construct() -> App<'static> {
    ConfigOpts::into_app().name("rename")
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: ConfigOpts = ConfigOpts::from_arg_matches(args);
    // Cannot use the `env` variable as we need a mutable copy.
    let mut config: Config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?
        .as_ref()
        .clone();

    let config_path = opts.config_path.as_str();
    let format = opts.format.as_str();

    // We replace `.` with `/` so the user can use `path.value.field` instead of forcing him
    // to use `path/value/field`. Since none of our keys have slashes or tildes in them it
    // won't be a problem.
    let mut config_path = config_path.replace(".", "/");
    // We change config path to starts with a `/` if it doesn't already. This is because
    // JSON pointers can be relative, but we don't have a place to start if is it.
    if !config_path.starts_with('/') {
        config_path.insert(0, '/');
    }

    if config_path == "/" {
        config_path.clear()
    }

    if let Some(arg_value) = Some(opts.value.as_str()) {
        // Try to parse the type of the value (which is a string from the arguments) as
        // JSON. By default we will just assume the type is string (if all parsing fails).
        let value = serde_json::from_str::<Value>(arg_value)
            .unwrap_or_else(|_| Value::String(arg_value.to_owned()));
        *config
            .get_mut_json()
            .pointer_mut(config_path.as_str())
            .ok_or(DfxError::ConfigPathDoesNotExist(config_path))? = value;
        config.save()
    } else if let Some(value) = config.get_json().pointer(config_path.as_str()) {
        match format {
            "text" => println!("{}", value),
            "json" => println!("{}", serde_json::to_string_pretty(value)?),
            _ => {}
        }
        Ok(())
    } else {
        Err(DfxError::ConfigPathDoesNotExist(config_path))
    }
}
