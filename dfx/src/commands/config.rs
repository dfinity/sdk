use crate::commands::CliResult;
use crate::config::dfinity::Config;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("config")
        .about("Configure options in the current DFINITY project.")
        .arg(
            Arg::with_name("option_name")
                .help("The name configuration option to set or read.")
                .required(true),
        )
        .arg(Arg::with_name("value").help(
            "The new value to set. If unspecified will output the current value in the config.",
        ))
}

pub fn exec(args: &ArgMatches<'_>) -> CliResult {
    let mut config = Config::from_current_dir()?;

    let option_name = args.value_of("option_name").unwrap();
    if let Some(value) = args.value_of("value") {
        let new_value = serde_json::from_str(value)?;
        *config.get_mut_json().pointer_mut(option_name).unwrap() = new_value;
        config.save()
    } else {
        if let Some(value) = config.get_json().pointer(option_name) {
            println!("{}", value);
        }
        Ok(())
    }
}
