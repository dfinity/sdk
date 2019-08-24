use crate::commands::CliResult;
use crate::config::dfinity::{Config, ConfigCanistersCanister};
use clap::{Arg, ArgMatches, SubCommand, App};
use crate::config::cache::binary_command;

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Build a canister code, or all canisters if no argument is passed.")
        .arg(
            Arg::with_name("canister")
                .help("The canister name to build.")
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let build_root = project_root.join(config.get_config().get_defaults().get_build().get_output("build/"));
    println!("{}", build_root.to_str().unwrap());

    match &config.get_config().canisters {
        Some(canisters) => {
            for (k, v) in canisters {
                let v: ConfigCanistersCanister = serde_json::from_value(v.to_owned())?;

                println!("Building {}...", k);
                match v.main {
                    Some(x) => {
                        let mut output_path = build_root.join(x.as_str());
                        output_path.set_extension("wasm");

                        std::fs::create_dir_all(output_path.parent().unwrap())?;
                        binary_command(&config, "asc")?
                            .arg(project_root.join(x.as_str()).into_os_string())
                            .arg("-o").arg(&output_path)
                            .output()?;
                    },
                    _ => {},
                }
            }
        },
        _ => {},
    }

    Ok(())
}
