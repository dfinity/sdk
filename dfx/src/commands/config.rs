use crate::config::dfinity::Config;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use serde_json::value::Value;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("config")
        .about("Configure options in the current DFINITY project.")
        .arg(
            Arg::with_name("config_path")
                .help("The name of the configuration option to set or read.")
                .required(true),
        )
        .arg(Arg::with_name("value").help(
            "The new value to set. If unspecified will output the current value in the config.",
        ))
}

pub fn exec<T>(_env: &T, args: &ArgMatches<'_>) -> DfxResult {
    // Cannot use the `env` variable as we need a mutable copy.
    let mut config = Config::from_current_dir()?;

    let config_path = args.value_of("config_path").unwrap();
    if let Some(value) = args.value_of("value") {
        // Try to parse the type of the value (which is a string). By default we will just assume
        // the type is string (if all parsing fails).
        if let Ok(new_value) = serde_json::from_str(value) {
            *config.get_mut_json().pointer_mut(config_path).unwrap() = new_value;
        } else {
            *config.get_mut_json().pointer_mut(config_path).unwrap() =
                Value::String(value.to_owned());
        }
        config.save()
    } else if let Some(value) = config.get_json().pointer(config_path) {
        println!("{}", value);
        Ok(())
    } else {
        Err(DfxError::ConfigPathDoesNotExist(config_path.to_owned()))
    }
}
