use crate::commands::CliResult;
use crate::config::Config;
use clap::{ArgMatches, SubCommand, Arg, App};


pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("config")
        .about("Configure options in the current DFINITY project.")
        .arg(
            Arg::with_name("option_name")
                .help("The name configuration option to set or read.")
                .required(true)
        )
        .arg(
            Arg::with_name("value")
                .help("The new value to set. If unspecified will output the current value in the config.")
        )
}

pub fn exec(args: &ArgMatches<'_>) -> CliResult {
    let mut config = Config::load_from(&std::env::current_dir()?)?;

    let option_name = args.value_of("option_name").unwrap();
    if args.is_present("value") {
        config.get_mut_value()[option_name] = serde_json::from_str(args.value_of("value").unwrap())?;
        config.save()
    } else {
        if let Some(value) = config.get_value().get(option_name) {
            println!("{}", value);
        }
    }
    Ok(())
}
