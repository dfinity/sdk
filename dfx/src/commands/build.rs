use crate::config::cache::binary_command_from_config;
use crate::config::dfinity::{Config, ConfigCanistersCanister};
use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Build a canister code, or all canisters if no argument is passed.")
        .arg(Arg::with_name("canister").help("The canister name to build."))
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let build_root = project_root.join(
        config
            .get_config()
            .get_defaults()
            .get_build()
            .get_output("build/"),
    );

    if let Some(canisters) = &config.get_config().canisters {
        for (k, v) in canisters {
            let v: ConfigCanistersCanister = serde_json::from_value(v.to_owned())?;

            println!("Building {}...", k);
            if let Some(x) = v.main {
                let input_as_path = project_root.join(x.as_str()).into_os_string();

                let mut output_wasm_path = build_root.join(x.as_str());
                let mut output_idl_path = output_wasm_path.clone();
                let mut output_js_path = output_wasm_path.clone();
                output_wasm_path.set_extension("wasm");
                output_idl_path.set_extension("did");
                output_js_path.set_extension("js");

                std::fs::create_dir_all(output_wasm_path.parent().unwrap())?;

                binary_command_from_config(&config, "asc")?
                    .arg(&input_as_path)
                    .arg("-o")
                    .arg(&output_wasm_path)
                    .output()?;
                binary_command_from_config(&config, "asc")?
                    .arg("--idl")
                    .arg(&input_as_path)
                    .arg("-o")
                    .arg(&output_idl_path)
                    .output()?;
                binary_command_from_config(&config, "didc")?
                    .arg("--js")
                    .arg(&output_idl_path)
                    .arg("-o")
                    .arg(&output_js_path)
                    .output()?;
            }
        }
    }

    Ok(())
}
