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

    match &config.get_config().canisters {
        Some(canisters) => {
            for (k, v) in canisters {
                let v: ConfigCanistersCanister = serde_json::from_value(v.to_owned())?;

                println!("Building {}...", k);
                match v.main {
                    Some(x) => {

                        let mut output_wasm_path = build_root.join(x.as_str());
                        let mut output_idl_path = output_wasm_path.clone();
                        let mut output_js_path = output_wasm_path.clone();
                        output_wasm_path.set_extension("wasm");
                        output_idl_path.set_extension("did");
                        output_js_path.set_extension("js");

                        std::fs::create_dir_all(output_wasm_path.parent().unwrap())?;

                        binary_command(&config, "asc")?
                            .arg(project_root.join(x.as_str()).into_os_string())
                            .arg("-o").arg(&output_wasm_path)
                            .output()?;
                        binary_command(&config, "didc")?
                            .arg("--js")
                            .arg(&output_idl_path)
                            .arg("-o").arg(&output_js_path)
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
