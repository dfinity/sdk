use crate::config::cache::binary_command;
use crate::config::dfinity::{Config, ConfigCanistersCanister};
use crate::lib::build::{build_file, watch_file_and_spin};
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use std::sync::Arc;

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Build a canister code, or all canisters if no argument is passed.")
        .arg(
            Arg::with_name("canister")
                .help("The canister name to build. By default builds all canisters."),
        )
        .arg(
            Arg::with_name("watch")
                .long("watch")
                .help("Watch input files and rebuild on changes.")
                .takes_value(false),
        )
}

fn just_build() -> DfxResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let output_root = project_root.join(
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
                let config: &'static Config = Box::leak(Box::new(config.clone()));
                let input_as_path = project_root.join(x.as_str());

                build_file(
                    &move |name| binary_command(config, name).map_err(DfxError::StdIo),
                    &input_as_path,
                    &output_root.join(x.as_str()),
                )?;
            }
        }
    }

    Ok(())
}

fn watch_and_build() -> DfxResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let output_root = project_root.join(
        config
            .get_config()
            .get_defaults()
            .get_build()
            .get_output("build/"),
    );

    if let Some(canisters) = &config.get_config().canisters {
        let config = config.clone();

        let multi = MultiProgress::new();
        multi.set_draw_target(ProgressDrawTarget::stderr());

        for (_, v) in canisters {
            let v: ConfigCanistersCanister = serde_json::from_value(v.to_owned())?;

            if let Some(x) = v.main {
                let input_as_path = project_root.join(x.as_str());

                let bar = Arc::new(multi.add(ProgressBar::new_spinner()));
                let config = Arc::new(config.clone());

                watch_file_and_spin(
                    bar,
                    Arc::new(move |name| {
                        binary_command(Arc::clone(&config).as_ref(), name).map_err(DfxError::StdIo)
                    }),
                    &input_as_path,
                    &output_root.join(x.as_str()),
                )?;
            }
        }

        multi.join()?;
        loop {}
    }

    Ok(())
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    if args.occurrences_of("watch") > 0 {
        watch_and_build()
    } else {
        just_build()
    }
}
